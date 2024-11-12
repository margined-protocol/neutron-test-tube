package testenv

import (
	"encoding/json"
	"strings"
	"time"

	// helpers

	// tendermint
	"cosmossdk.io/log"
	sdkmath "cosmossdk.io/math"

	// adminmodule
	adminmoduletypes "github.com/cosmos/admin-module/v2/x/adminmodule/types"

	// cometbft
	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	tmtypes "github.com/cometbft/cometbft/types"
	dbm "github.com/cosmos/cosmos-db"

	// cosmos sdk
	"github.com/cosmos/cosmos-sdk/baseapp"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/server"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	slashingtypes "github.com/cosmos/cosmos-sdk/x/slashing/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"

	// interchain security
	ccvconsumertypes "github.com/cosmos/interchain-security/v5/x/ccv/consumer/types"
	ccvtypes "github.com/cosmos/interchain-security/v5/x/ccv/types"

	// wasmd
	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"

	// neutron
	"github.com/neutron-org/neutron/v5/app"
	dexmoduletypes "github.com/neutron-org/neutron/v5/x/dex/types"
	tokenfactorytypes "github.com/neutron-org/neutron/v5/x/tokenfactory/types"

	// slinky
	compression "github.com/skip-mev/slinky/abci/strategies/codec"
	"github.com/skip-mev/slinky/abci/testutils"
	slinkytypes "github.com/skip-mev/slinky/pkg/types"
	marketmaptypes "github.com/skip-mev/slinky/x/marketmap/types"
	oraclekeeper "github.com/skip-mev/slinky/x/oracle/keeper"
	oracletypes "github.com/skip-mev/slinky/x/oracle/types"
)

type TestEnv struct {
	App                *app.App
	Ctx                sdk.Context
	ParamTypesRegistry ParamTypeRegistry
	ValPrivs           secp256k1.PrivKey
	Validator          []byte
	NodeHome           string
}

type DebugAppOptions map[string]interface{}

func (m DebugAppOptions) Get(key string) interface{} {
	v, ok := m[key]
	if !ok {
		return nil
	}

	return v
}

func NewDebugAppOptionsWithFlagHome() servertypes.AppOptions {
	return DebugAppOptions{
		server.FlagTrace: true,
	}
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
		0,
		encCfg,
		NewDebugAppOptionsWithFlagHome(),
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

	consensusParams := simtestutil.DefaultConsensusParams
	consensusParams.Block = &tmproto.BlockParams{
		MaxBytes: 22020096,
		MaxGas:   -1,
	}
	consensusParams.Abci = &tmproto.ABCIParams{
		VoteExtensionsEnableHeight: 2,
	}

	// replace sdk.DefaultDenom with "untrn", a bit of a hack, needs improvement
	stateBytes = []byte(strings.Replace(string(stateBytes), "\"stake\"", "\"untrn\"", -1))

	appInstance.InitChain(
		&abci.RequestInitChain{
			Validators:      []abci.ValidatorUpdate{},
			ConsensusParams: consensusParams,
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
	valAcc := authtypes.NewBaseAccountWithAddress(pubKey.Address().Bytes())

	// generate genesis account
	senderPrivKey := secp256k1.GenPrivKey()
	senderPrivKey.PubKey().Address()
	acc := authtypes.NewBaseAccountWithAddress(senderPrivKey.PubKey().Address().Bytes())

	//////////////////////
	validatorBalance := banktypes.Balance{
		Address: valAcc.GetAddress().String(),
		Coins:   sdk.NewCoins(sdk.NewCoin("untrn", sdkmath.NewInt(1000000000000000000))),
	}
	balances := []banktypes.Balance{validatorBalance}
	genesisState := app.NewDefaultGenesisState(appInstance.AppCodec())
	genAccs := []authtypes.GenesisAccount{acc, valAcc}
	authGenesis := authtypes.NewGenesisState(authtypes.DefaultParams(), genAccs)
	genesisState[authtypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(authGenesis)

	// set adminmodule genesis state
	adminGen := adminmoduletypes.GenesisState{
		Admins: []string{valAcc.Address},
	}
	genesisState[adminmoduletypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(&adminGen)

	// set marketmap genesis state
	marketmapGen := marketmaptypes.GenesisState{
		Params: marketmaptypes.Params{
			MarketAuthorities: []string{valAcc.Address},
			Admin:             valAcc.Address,
		},
	}
	genesisState[marketmaptypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(&marketmapGen)

	// set oracle genesis state
	oracleGen := oracletypes.GenesisState{
		CurrencyPairGenesis: []oracletypes.CurrencyPairGenesis{
			{
				CurrencyPair: slinkytypes.CurrencyPair{
					Base:  "ATOM",
					Quote: "USDT",
				},
				CurrencyPairPrice: &oracletypes.QuotePrice{Price: sdkmath.NewInt(4480000)},
				Nonce:             0,
				Id:                0,
			},
		},
		NextId: 1,
	}
	genesisState[oracletypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(&oracleGen)

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

	// initialValset := []abci.ValidatorUpdate{{PubKey: tmProtoPublicKey, Power: 100}}
	ccvGenesis := ccvconsumertypes.DefaultGenesisState()
	ccvGenesis.Params.Enabled = true
	ccvGenesis.PreCCV = false
	ccvGenesis.Provider = ccvtypes.ProviderInfo{
		InitialValSet: initValPowers,
	}
	genesisState[ccvconsumertypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(ccvGenesis)

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

func CreateExtendedVoteInfo(val secp256k1.PrivKey, prices map[uint64][]byte) []byte {
	ca := sdk.ConsAddress(val.PubKey().Address())

	// Create the vote extensions handler that will be used to extend and verify
	// vote extensions (i.e. oracle data).
	veCodec := compression.NewCompressionVoteExtensionCodec(
		compression.NewDefaultVoteExtensionCodec(),
		compression.NewZLibCompressor(),
	)
	extCommitCodec := compression.NewCompressionExtendedCommitCodec(
		compression.NewDefaultExtendedCommitCodec(),
		compression.NewZStdCompressor(),
	)

	vote, err := testutils.CreateExtendedVoteInfo(
		ca,
		prices,
		veCodec,
	)
	requireNoErr(err)

	_, extCommitInfoBz, err := testutils.CreateExtendedCommitInfo(
		[]abci.ExtendedVoteInfo{vote},
		extCommitCodec,
	)
	requireNoErr(err)

	return extCommitInfoBz
}

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

func GetCurrentPriceAndPairMapping(ctx sdk.Context, oracle oraclekeeper.Keeper, base, quote string) (sdkmath.Int, uint64, error) {
	ccyPair := slinkytypes.CurrencyPair{
		Base:  base,
		Quote: quote,
	}

	pairs, err := oracle.GetCurrencyPairMapping(ctx)
	if err != nil {
		return sdkmath.ZeroInt(), 0, err
	}

	pairIndex := uint64(0)
	for idx, pair := range pairs {
		if pair.Base == base && pair.Quote == quote {
			pairIndex = idx
		}
	}

	res, err := oracle.GetPriceForCurrencyPair(ctx, ccyPair)
	if err != nil {
		return sdkmath.ZeroInt(), pairIndex, nil
	}

	return res.Price, pairIndex, nil
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
