package testenv

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	// helpers

	// tendermint
	"cosmossdk.io/log"
	sdkmath "cosmossdk.io/math"

	// dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	tmtypes "github.com/cometbft/cometbft/types"
	dbm "github.com/cosmos/cosmos-db"
	"github.com/cosmos/cosmos-sdk/baseapp"

	// cosmos sdk
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/server"

	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"

	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"

	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	slashingtypes "github.com/cosmos/cosmos-sdk/x/slashing/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"

	// wasmd

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"

	// neutron
	"github.com/neutron-org/neutron/v4/app"
	dexmoduletypes "github.com/neutron-org/neutron/v4/x/dex/types"
	tokenfactorytypes "github.com/neutron-org/neutron/v4/x/tokenfactory/types"
)

type TestEnv struct {
	App                *app.App
	Ctx                sdk.Context
	ParamTypesRegistry ParamTypeRegistry
	ValPrivs           []*secp256k1.PrivKey
	Validator          []byte
	NodeHome           string
}

// DebugAppOptions is a stub implementing AppOptions
type DebugAppOptions struct{}

// Get implements AppOptions
func (ao DebugAppOptions) Get(o string) interface{} {
	if o == server.FlagTrace {
		return true
	}
	return nil
}

func NewNeutronApp(nodeHome string) *app.App {
	db := dbm.NewMemDB()

	encCfg := app.MakeEncodingConfig()
	var emptyWasmOpts []wasmkeeper.Option

	return app.New(
		log.NewNopLogger(),
		db,
		nil,
		true,
		map[int64]bool{},
		nodeHome,
		5,
		encCfg,
		DebugAppOptions{},
		emptyWasmOpts,
		baseapp.SetChainID("neutron-666"),
	)
}

func InitChain(appInstance *app.App) (sdk.Context, secp256k1.PrivKey) {
	sdk.DefaultBondDenom = "untrn"
	genesisState, valPriv := GenesisStateWithValSet(appInstance)

	encCfg := app.MakeEncodingConfig()

	// Set up Wasm genesis state
	wasmGen := wasmtypes.GenesisState{
		Params: wasmtypes.Params{
			// Allow store code without gov
			CodeUploadAccess:             wasmtypes.AllowEverybody,
			InstantiateDefaultPermission: wasmtypes.AccessTypeEverybody,
		},
	}
	genesisState[wasmtypes.ModuleName] = encCfg.Marshaler.MustMarshalJSON(&wasmGen)

	// set staking genesis state
	stakingGenesisState := stakingtypes.GenesisState{}
	appInstance.AppCodec().UnmarshalJSON(genesisState[stakingtypes.ModuleName], &stakingGenesisState)

	stateBytes, err := json.MarshalIndent(genesisState, "", " ")

	requireNoErr(err)

	concensusParams := simtestutil.DefaultConsensusParams
	concensusParams.Block = &tmproto.BlockParams{
		MaxBytes: 22020096,
		MaxGas:   -1,
	}

	// replace sdk.DefaultDenom with "untrn", a bit of a hack, needs improvement
	stateBytes = []byte(strings.Replace(string(stateBytes), "\"stake\"", "\"untrn\"", -1))

	appInstance.InitChain(
		&abci.RequestInitChain{
			Validators:      []abci.ValidatorUpdate{},
			ConsensusParams: concensusParams,
			AppStateBytes:   stateBytes,
			ChainId:         "neutron-666",
		},
	)

	ctx := appInstance.NewUncachedContext(false, tmproto.Header{Height: 0, ChainID: "neutron-666", Time: time.Now().UTC()})

	// for each stakingGenesisState.Validators
	for _, validator := range stakingGenesisState.Validators {
		consAddr, err := validator.GetConsAddr()
		requireNoErr(err)
		signingInfo := slashingtypes.NewValidatorSigningInfo(
			consAddr,
			ctx.BlockHeight(),
			0,
			time.Unix(0, 0),
			false,
			0,
		)
		appInstance.SlashingKeeper.SetValidatorSigningInfo(ctx, consAddr, signingInfo)
	}

	return ctx, valPriv
}

func GenesisStateWithValSet(appInstance *app.App) (app.GenesisState, secp256k1.PrivKey) {
	privVal := NewPV()
	pubKey, _ := privVal.GetPubKey()
	validator := tmtypes.NewValidator(pubKey, 1)
	valSet := tmtypes.NewValidatorSet([]*tmtypes.Validator{validator})

	// generate genesis account
	senderPrivKey := secp256k1.GenPrivKey()
	senderPrivKey.PubKey().Address()
	acc := authtypes.NewBaseAccountWithAddress(senderPrivKey.PubKey().Address().Bytes())

	//////////////////////
	balances := []banktypes.Balance{}
	genesisState := app.NewDefaultGenesisState(appInstance.AppCodec())
	genAccs := []authtypes.GenesisAccount{acc}
	authGenesis := authtypes.NewGenesisState(authtypes.DefaultParams(), genAccs)
	genesisState[authtypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(authGenesis)

	validators := make([]stakingtypes.Validator, 0, len(valSet.Validators))
	delegations := make([]stakingtypes.Delegation, 0, len(valSet.Validators))

	bondAmt := sdk.DefaultPowerReduction
	initValPowers := []abci.ValidatorUpdate{}

	for _, val := range valSet.Validators {
		pk, _ := cryptocodec.FromCmtPubKeyInterface(val.PubKey)
		pkAny, _ := codectypes.NewAnyWithValue(pk)
		validator := stakingtypes.Validator{
			OperatorAddress:   sdk.ValAddress(val.Address).String(),
			ConsensusPubkey:   pkAny,
			Jailed:            false,
			Status:            stakingtypes.Bonded,
			Tokens:            bondAmt,
			DelegatorShares:   sdkmath.LegacyOneDec(),
			Description:       stakingtypes.Description{},
			UnbondingHeight:   int64(0),
			UnbondingTime:     time.Unix(0, 0).UTC(),
			Commission:        stakingtypes.NewCommission(sdkmath.LegacyZeroDec(), sdkmath.LegacyZeroDec(), sdkmath.LegacyZeroDec()),
			MinSelfDelegation: sdkmath.ZeroInt(),
		}
		validators = append(validators, validator)
		delegations = append(delegations, stakingtypes.NewDelegation(genAccs[0].String(), val.Address.String(), sdkmath.LegacyOneDec()))

		// add initial validator powers so consumer InitGenesis runs correctly
		pub, _ := val.ToProto()
		initValPowers = append(initValPowers, abci.ValidatorUpdate{
			Power:  val.VotingPower,
			PubKey: pub.PubKey,
		})
	}
	// set validators and delegations
	stakingGenesis := stakingtypes.NewGenesisState(stakingtypes.DefaultParams(), validators, delegations)
	genesisState[stakingtypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(stakingGenesis)

	totalSupply := sdk.NewCoins()
	for _, b := range balances {
		// add genesis acc tokens to total supply
		totalSupply = totalSupply.Add(b.Coins...)
	}

	for range delegations {
		// add delegated tokens to total supply
		totalSupply = totalSupply.Add(sdk.NewCoin(sdk.DefaultBondDenom, bondAmt))
	}

	// add bonded amount to bonded pool module account
	balances = append(balances, banktypes.Balance{
		Address: authtypes.NewModuleAddress(stakingtypes.BondedPoolName).String(),
		Coins:   sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, bondAmt)},
	})

	// update total supply
	bankGenesis := banktypes.NewGenesisState(
		banktypes.DefaultGenesisState().Params,
		balances,
		totalSupply,
		[]banktypes.Metadata{},
		[]banktypes.SendEnabled{},
	)
	genesisState[banktypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(bankGenesis)

	_, err := tmtypes.PB2TM.ValidatorUpdates(initValPowers)
	if err != nil {
		panic("failed to get vals")
	}

	return genesisState, secp256k1.PrivKey{Key: privVal.PrivKey.Bytes()}
}

func (env *TestEnv) BeginNewBlock(executeNextEpoch bool, timeIncreaseSeconds uint64) {
	newBlockTime := env.Ctx.BlockTime().Add(time.Duration(timeIncreaseSeconds) * time.Second)

	newCtx := env.Ctx.WithBlockTime(newBlockTime).WithBlockHeight(env.Ctx.BlockHeight() + 1)
	env.Ctx = newCtx

	env.App.BeginBlocker(newCtx)

	env.Ctx = env.App.NewContext(false)
}

// func (env *TestEnv) GetValidatorAddresses() []string {
// 	validators := env.App.StakingKeeper.GetAllValidators(env.Ctx)
// 	var addresses []string
// 	for _, validator := range validators {
// 		addresses = append(addresses, validator.OperatorAddress)
// 	}

// 	return addresses
// }

func (env *TestEnv) GetValidatorPrivateKey() []byte {
	return env.Validator
}

func (env *TestEnv) SetDefaultValidator(consAddr sdk.ConsAddress) {
	signingInfo := slashingtypes.NewValidatorSigningInfo(
		consAddr,
		env.Ctx.BlockHeight(),
		0,
		time.Unix(0, 0),
		false,
		0,
	)
	env.App.SlashingKeeper.SetValidatorSigningInfo(env.Ctx, consAddr, signingInfo)
}

func (env *TestEnv) FundAccount(ctx sdk.Context, bankKeeper bankkeeper.Keeper, addr sdk.AccAddress, amounts sdk.Coins) error {
	if err := bankKeeper.MintCoins(ctx, dexmoduletypes.ModuleName, amounts); err != nil {
		return err
	}

	return bankKeeper.SendCoinsFromModuleToAccount(ctx, dexmoduletypes.ModuleName, addr, amounts)
}

func (env *TestEnv) SetupParamTypes() {
	pReg := env.ParamTypesRegistry
	pReg.RegisterParamSet(&tokenfactorytypes.Params{})

}

func requireNoErr(err error) {
	if err != nil {
		panic(err)
	}
}

func requireNoNil(name string, nilable any) {
	if nilable == nil {
		panic(fmt.Sprintf("%s must not be nil", name))
	}
}

func requireTrue(name string, b bool) {
	if !b {
		panic(fmt.Sprintf("%s must be true", name))
	}
}
