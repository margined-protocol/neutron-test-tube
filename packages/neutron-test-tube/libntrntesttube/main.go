package main

import "C"

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"sync"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/cosmos/gogoproto/proto"
	"github.com/margined-protocol/test-tube/neutron-test-tube/result"
	"github.com/margined-protocol/test-tube/neutron-test-tube/testenv"
	"github.com/pkg/errors"

	sdkmath "cosmossdk.io/math"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
)

var (
	envCounter  uint64 = 0
	envRegister        = sync.Map{}
	mu          sync.Mutex
)

type Price struct {
	Base  string `json:"base"`
	Quote string `json:"quote"`
	Price int    `json:"price"`
}

//export InitTestEnv
func InitTestEnv() uint64 { // Temp fix for concurrency issue
	mu.Lock()
	defer mu.Unlock()

	// temp: suppress noise from stdout
	os.Stdout = nil

	envCounter += 1
	id := envCounter

	nodeHome, err := os.MkdirTemp("", ".neutron-test-tube-temp-")
	if err != nil {
		panic(err)
	}

	// set up the validator
	env := new(testenv.TestEnv)
	env.App = testenv.NewNeutronApp(nodeHome)
	env.NodeHome = nodeHome
	env.ParamTypesRegistry = *testenv.NewParamTypeRegistry()

	ctx, valPriv := testenv.InitChain(env.App)

	env.Ctx = ctx
	env.ValPrivs = secp256k1.PrivKey{Key: valPriv.Bytes()}
	env.Validator = valPriv.Bytes()

	env.SetupParamTypes()

	// Allow testing unoptimized contract
	wasmtypes.MaxWasmSize = 1024 * 1024 * 1024 * 1024 * 1024

	newBlockTime := env.Ctx.BlockTime().Add(time.Duration(3) * time.Second)
	newCtx := env.Ctx.WithBlockTime(newBlockTime).WithBlockHeight(env.Ctx.BlockHeight() + 1)
	env.Ctx = newCtx

	reqFinalizeBlock := abci.RequestFinalizeBlock{Height: env.Ctx.BlockHeight(), Txs: [][]byte{}, Time: newBlockTime}

	env.App.FinalizeBlock(&reqFinalizeBlock)
	env.App.Commit()

	envRegister.Store(id, *env)

	return id
}

//export CleanUp
func CleanUp(envId uint64) {
	env := loadEnv(envId)
	err := os.RemoveAll(env.NodeHome)
	if err != nil {
		panic(err)
	}
	envRegister.Delete(envId)
}

//export InitAccount
func InitAccount(envId uint64, coinsJson string) *C.char {
	env := loadEnv(envId)
	var coins sdk.Coins

	if err := json.Unmarshal([]byte(coinsJson), &coins); err != nil {
		panic(err)
	}

	priv := secp256k1.GenPrivKey()
	accAddr := sdk.AccAddress(priv.PubKey().Address())
	for _, coin := range coins {
		// create denom if not exist
		_, hasDenomMetaData := env.App.BankKeeper.GetDenomMetaData(env.Ctx, coin.Denom)
		if !hasDenomMetaData {
			denomMetaData := banktypes.Metadata{
				DenomUnits: []*banktypes.DenomUnit{{
					Denom:    coin.Denom,
					Exponent: 0,
				}},
				Base: coin.Denom,
			}

			env.App.BankKeeper.SetDenomMetaData(env.Ctx, denomMetaData)
		}

	}

	err := env.FundAccount(env.Ctx, env.App.BankKeeper, accAddr, coins)
	if err != nil {
		panic(errors.Wrapf(err, "Failed to fund account"))
	}

	base64Priv := base64.StdEncoding.EncodeToString(priv.Bytes())

	envRegister.Store(envId, env)

	return C.CString(base64Priv)
}

// Core function to adjust block time and finalize
func finalizeWithTime(envId uint64, txBytes [][]byte, seconds uint64) *C.char {

	env := loadEnv(envId)
	mu.Lock()
	defer mu.Unlock()

	// Update context with new block time and height
	newBlockTime := env.Ctx.BlockTime().Add(time.Duration(seconds) * time.Second)
	newCtx := env.Ctx.WithBlockTime(newBlockTime).WithBlockHeight(env.Ctx.BlockHeight() + 1)
	env.Ctx = newCtx

	reqFinalizeBlock := &abci.RequestFinalizeBlock{
		Height: env.Ctx.BlockHeight(),
		Txs:    txBytes,
		Time:   newBlockTime,
	}

	// Finalize the block
	res, err := env.App.FinalizeBlock(reqFinalizeBlock)
	if err != nil {
		panic(err)
	}
	_, err = env.App.Commit()
	if err != nil {
		panic(err)
	}

	// Marshal result and update environment registry
	bz, err := proto.Marshal(res)
	if err != nil {
		panic(err)
	}
	envRegister.Store(envId, env)

	return encodeBytesResultBytes(bz)
}

// Helper function to create transaction bytes based on block height
func getTxBytes(env *testenv.TestEnv, reqDeliverTxBytes []byte, additionalBytes []byte) [][]byte {
	if env.Ctx.BlockHeight() < 2 {
		return [][]byte{reqDeliverTxBytes}
	}
	return [][]byte{additionalBytes, reqDeliverTxBytes}
}

//export IncreaseTime
func IncreaseTime(envId uint64, seconds uint64) {
	env := loadEnv(envId)

	finalizeWithTime(envId, getTxBytes(&env, nil, nil), seconds)
}

//export FinalizeBlock
func FinalizeBlock(envId uint64, base64ReqDeliverTx string) *C.char {
	env := loadEnv(envId)

	reqDeliverTxBytes, err := base64.StdEncoding.DecodeString(base64ReqDeliverTx)
	if err != nil {
		panic(err)
	}

	return finalizeWithTime(envId, getTxBytes(&env, reqDeliverTxBytes, nil), 3)
}

//export SetSlinkyPrices
func SetSlinkyPrices(envId uint64, pricesJson string) {
	env := loadEnv(envId)

	prices := parsePrices(pricesJson)
	slinkyPrices := calculateSlinkyPrices(&env, prices)

	extCommitInfoBz := testenv.CreateExtendedVoteInfo(env.ValPrivs, slinkyPrices)

	finalizeWithTime(envId, getTxBytes(&env, nil, extCommitInfoBz), 3)
}

// Helper to parse JSON prices into Price struct array
func parsePrices(pricesJson string) []Price {
	var prices []Price
	if err := json.Unmarshal([]byte(pricesJson), &prices); err != nil {
		panic(err)
	}
	return prices
}

// Helper to calculate slinky prices
func calculateSlinkyPrices(env *testenv.TestEnv, prices []Price) map[uint64][]byte {
	slinkyPrices := map[uint64][]byte{}
	for _, price := range prices {
		currentPrice, idx, err := testenv.GetCurrentPriceAndPairMapping(env.Ctx, *env.App.OracleKeeper, price.Base, price.Quote)
		if err != nil {
			panic(err)
		}
		newPrice := sdkmath.NewIntFromBigInt(big.NewInt(int64(price.Price)))
		delta := newPrice.Sub(currentPrice)
		encodedDelta, err := big.NewInt(delta.Int64()).GobEncode()
		if err != nil {
			panic(err)
		}
		slinkyPrices[idx] = encodedDelta
	}
	return slinkyPrices
}

//export WasmSudo
func WasmSudo(envId uint64, bech32Address, msgJson string) *C.char {
	env := loadEnv(envId)
	// Temp fix for concurrency issue
	mu.Lock()
	defer mu.Unlock()

	accAddr, err := sdk.AccAddressFromBech32(bech32Address)
	if err != nil {
		panic(err)
	}

	msgBytes := []byte(msgJson)

	res, err := env.App.WasmKeeper.Sudo(env.Ctx, accAddr, msgBytes)
	if err != nil {
		return encodeErrToResultBytes(result.ExecuteError, err)
	}

	envRegister.Store(envId, env)

	return encodeBytesResultBytes(res)
}

//export Query
func Query(envId uint64, path, base64QueryMsgBytes string) *C.char {
	env := loadEnv(envId)
	queryMsgBytes, err := base64.StdEncoding.DecodeString(base64QueryMsgBytes)
	if err != nil {
		panic(err)
	}

	req := abci.RequestQuery{}
	req.Data = queryMsgBytes

	route := env.App.GRPCQueryRouter().Route(path)
	if route == nil {
		err := errors.New("No route found for `" + path + "`")
		return encodeErrToResultBytes(result.QueryError, err)
	}
	res, err := route(env.Ctx, &req)

	if err != nil {
		return encodeErrToResultBytes(result.QueryError, err)
	}

	return encodeBytesResultBytes(res.Value)
}

//export GetBlockTime
func GetBlockTime(envId uint64) int64 {
	env := loadEnv(envId)
	return env.Ctx.BlockTime().UnixNano()
}

//export GetBlockHeight
func GetBlockHeight(envId uint64) int64 {
	env := loadEnv(envId)
	return env.Ctx.BlockHeight()
}

//export AccountSequence
func AccountSequence(envId uint64, bech32Address string) uint64 {
	env := loadEnv(envId)

	addr, err := sdk.AccAddressFromBech32(bech32Address)

	if err != nil {
		panic(err)
	}

	seq, err := env.App.AccountKeeper.GetSequence(env.Ctx, addr)

	if err != nil {
		panic(err)
	}

	return seq
}

//export AccountNumber
func AccountNumber(envId uint64, bech32Address string) uint64 {
	env := loadEnv(envId)

	addr, err := sdk.AccAddressFromBech32(bech32Address)

	if err != nil {
		panic(err)
	}

	acc := env.App.AccountKeeper.GetAccount(env.Ctx, addr)
	return acc.GetAccountNumber()
}

//export Simulate
func Simulate(envId uint64, base64TxBytes string) *C.char { // => base64GasInfo
	env := loadEnv(envId)
	// Temp fix for concurrency issue
	mu.Lock()
	defer mu.Unlock()

	txBytes, err := base64.StdEncoding.DecodeString(base64TxBytes)
	if err != nil {
		panic(err)
	}

	gasInfo, _, err := env.App.Simulate(txBytes)

	if err != nil {
		return encodeErrToResultBytes(result.ExecuteError, err)
	}

	bz, err := proto.Marshal(&gasInfo)
	if err != nil {
		panic(err)
	}

	return encodeBytesResultBytes(bz)
}

//export SetParamSet
func SetParamSet(envId uint64, subspaceName, base64ParamSetBytes string) *C.char {
	env := loadEnv(envId)

	// Temp fix for concurrency issue
	mu.Lock()
	defer mu.Unlock()

	paramSetBytes, err := base64.StdEncoding.DecodeString(base64ParamSetBytes)
	if err != nil {
		panic(err)
	}

	subspace, ok := env.App.ParamsKeeper.GetSubspace(subspaceName)
	if !ok {
		err := errors.New("No subspace found for `" + subspaceName + "`")
		return encodeErrToResultBytes(result.ExecuteError, err)
	}

	pReg := env.ParamTypesRegistry

	any := codectypes.Any{}
	err = proto.Unmarshal(paramSetBytes, &any)

	if err != nil {
		return encodeErrToResultBytes(result.ExecuteError, err)
	}

	pset, err := pReg.UnpackAny(&any)

	if err != nil {
		return encodeErrToResultBytes(result.ExecuteError, err)
	}

	subspace.SetParamSet(env.Ctx, pset)

	// return empty bytes if no error
	return encodeBytesResultBytes([]byte{})
}

//export GetParamSet
func GetParamSet(envId uint64, subspaceName, typeUrl string) *C.char {
	env := loadEnv(envId)

	subspace, ok := env.App.ParamsKeeper.GetSubspace(subspaceName)
	if !ok {
		err := errors.New("No subspace found for `" + subspaceName + "`")
		return encodeErrToResultBytes(result.ExecuteError, err)
	}

	pReg := env.ParamTypesRegistry
	pset, ok := pReg.GetEmptyParamsSet(typeUrl)

	if !ok {
		err := errors.New("No param set found for `" + typeUrl + "`")
		return encodeErrToResultBytes(result.ExecuteError, err)
	}
	subspace.GetParamSet(env.Ctx, pset)

	bz, err := proto.Marshal(pset)

	if err != nil {
		panic(err)
	}

	return encodeBytesResultBytes(bz)
}

//export GetValidatorAddress
func GetValidatorAddress(envId uint64, n int32) *C.char {
	// env := loadEnv(envId)
	// return C.CString(env.GetValidatorAddresses()[n])
	return C.CString("")
}

//export GetValidatorPrivateKey
func GetValidatorPrivateKey(envId uint64) *C.char {
	env := loadEnv(envId)
	priv := env.GetValidatorPrivateKey()

	base64Priv := base64.StdEncoding.EncodeToString(priv)

	return C.CString(base64Priv)
}

// ========= utils =========

func loadEnv(envId uint64) testenv.TestEnv {
	item, ok := envRegister.Load(envId)
	env := testenv.TestEnv(item.(testenv.TestEnv))
	if !ok {
		panic(fmt.Sprintf("env not found: %d", envId))
	}
	return env
}

func encodeErrToResultBytes(code byte, err error) *C.char {
	return C.CString(result.EncodeResultFromError(code, err))
}

func encodeBytesResultBytes(bytes []byte) *C.char {
	return C.CString(result.EncodeResultFromOk(bytes))
}

// must define main for ffi build
func main() {}
