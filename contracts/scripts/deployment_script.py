"""This example shows how to deploy a compiled contract to the Stellar network.
"""
import time
import os
from stellar_sdk import Keypair, Network, SorobanServer, StrKey, TransactionBuilder, scval, InvokeHostFunction
from stellar_sdk import xdr as stellar_xdr
from stellar_sdk.exceptions import PrepareTransactionException
from stellar_sdk.soroban_rpc import GetTransactionStatus, SendTransactionStatus
from stellar_sdk.auth import authorize_entry

SECRET_KEY = "SB46364SGIGPEQOLRXL6RTVDP4X2HBIMSNPIG246GAQC7VHHGHBOEV4M"
SUPER_ADMIN = "GB6A2R4B7MSB7HDD56DC4KIUCML3QGF2IT4JLTFHJNMHGGCJOVS3TELN"
STABLE_COIN = "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"

# TODO: You need to replace the following parameters according to the actual situation
rpc_server_url = "https://soroban-testnet.stellar.org:443"
network_passphrase = Network.TESTNET_NETWORK_PASSPHRASE

def deploy_contract(contract_file_path, secret):
    kp = Keypair.from_secret(secret)
    soroban_server = SorobanServer(rpc_server_url)
    print("uploading contract...")
    source = soroban_server.load_account(kp.public_key)
    with open(contract_file_path, "rb") as f:
        contract_bin = f.read()

    tx = (
        TransactionBuilder(source, network_passphrase)
        .set_timeout(300)
        .append_upload_contract_wasm_op(
            contract=contract_file_path,  # the path to the contract, or binary data
        )
        .build()
    )

    try:
        tx = soroban_server.prepare_transaction(tx)
    except PrepareTransactionException as e:
        print(f"Got exception: {e.simulate_transaction_response}")
        raise e

    tx.sign(kp)
    send_transaction_data = soroban_server.send_transaction(tx)
    if send_transaction_data.status != SendTransactionStatus.PENDING:
        raise Exception("send transaction failed")

    while True:
        print("waiting for transaction to be confirmed...")
        time.sleep(8)
        get_transaction_data = soroban_server.get_transaction(send_transaction_data.hash)
        if get_transaction_data.status != GetTransactionStatus.NOT_FOUND:
            break
        time.sleep(3)


    wasm_id = None
    if get_transaction_data.status == GetTransactionStatus.SUCCESS:
        assert get_transaction_data.result_meta_xdr is not None
        transaction_meta = stellar_xdr.TransactionMeta.from_xdr(
            get_transaction_data.result_meta_xdr
        )
        wasm_id = transaction_meta.v3.soroban_meta.return_value.bytes.sc_bytes.hex()  # type: ignore
        print(f"wasm id: {wasm_id}")
    else:
        print(f"Transaction failed: {get_transaction_data.result_xdr}")

    assert wasm_id, "wasm id should not be empty"

    print("creating contract...")

    source = soroban_server.load_account(
        kp.public_key
    )  # refresh source account, because the current SDK will increment the sequence number by one after building a transaction

    tx = (
        TransactionBuilder(source, network_passphrase)
        .set_timeout(300)
        .append_create_contract_op(wasm_id=wasm_id, address=kp.public_key)
        .build()
    )

    try:
        tx = soroban_server.prepare_transaction(tx)
    except PrepareTransactionException as e:
        print(f"Got exception: {e.simulate_transaction_response}")
        raise e

    tx.sign(kp)

    send_transaction_data = soroban_server.send_transaction(tx)
    time.sleep(5)
    if send_transaction_data.status != SendTransactionStatus.PENDING:
        raise Exception("send transaction failed")

    while True:
        print("waiting for transaction to be confirmed...")
        time.sleep(10)
        get_transaction_data = soroban_server.get_transaction(send_transaction_data.hash)
        if get_transaction_data.status != GetTransactionStatus.NOT_FOUND:
            break
        time.sleep(3)


    if get_transaction_data.status == GetTransactionStatus.SUCCESS:
        assert get_transaction_data.result_meta_xdr is not None
        transaction_meta = stellar_xdr.TransactionMeta.from_xdr(
            get_transaction_data.result_meta_xdr
        )
        result = transaction_meta.v3.soroban_meta.return_value.address.contract_id.hash  # type: ignore
        contract_id = StrKey.encode_contract(result)
        print(f"contract id: {contract_id}")
        return contract_id
    else:
        print(f"Transaction failed: {get_transaction_data.result_xdr}")


def send_tx(contract_id, secret, func_name, args):
    # https://github.com/stellar/soroban-examples/tree/v0.6.0/auth
    tx_submitter_kp = Keypair.from_secret(secret)
    soroban_server = SorobanServer(rpc_server_url)
    tx_submitter_kp = Keypair.from_secret(secret)
    # op_invoker_kp = Keypair.from_secret(
    #     "SAEZSI6DY7AXJFIYA4PM6SIBNEYYXIEM2MSOTHFGKHDW32MBQ7KVO6EN"
    # )

    source = soroban_server.load_account(tx_submitter_kp.public_key)
    tx = (
        TransactionBuilder(source, network_passphrase, base_fee=50000)
        .add_time_bounds(0, 0)
        .append_invoke_contract_function_op(
            contract_id=contract_id,
            function_name=func_name,
            parameters=args,
        )
        .build()
    )

    try:
        simulate_resp = soroban_server.simulate_transaction(tx)
        time.sleep(10)
        tx = soroban_server.prepare_transaction(tx, simulate_resp)
    except PrepareTransactionException as e:
        print(f"Got exception: {e.simulate_transaction_response}")
        raise e

    tx.sign(tx_submitter_kp)
    print(f"Signed XDR:\n{tx.to_xdr()}")

    send_transaction_data = soroban_server.send_transaction(tx)
    print(f"sent transaction: {send_transaction_data}")
    if send_transaction_data.status != SendTransactionStatus.PENDING:
        raise Exception("send transaction failed")

    while True:
        print("waiting for transaction to be confirmed...")
        time.sleep(10)
        get_transaction_data = soroban_server.get_transaction(send_transaction_data.hash)
        if get_transaction_data.status != GetTransactionStatus.NOT_FOUND:
            break
        time.sleep(3)

    print(f"transaction: {get_transaction_data}")

    if get_transaction_data.status == GetTransactionStatus.SUCCESS:
        assert get_transaction_data.result_meta_xdr is not None
        transaction_meta = stellar_xdr.TransactionMeta.from_xdr(
            get_transaction_data.result_meta_xdr
        )
        result = transaction_meta.v3.soroban_meta.return_value.u32  # type: ignore
        print(f"Function result: {result}")
    else:
        print(f"Transaction failed: {get_transaction_data.result_xdr}")
        
path = [
"../campaign_management/target/wasm32-unknown-unknown/release/campaign_management.wasm",
"../registry/target/wasm32-unknown-unknown/release/registry.wasm",
"../issuance_management/target/wasm32-unknown-unknown/release/issuance_management.wasm"
]

# CONTRACT INITIALIZATION
function_call = {"registry.wasm": 
[
    {"initialize" : "SUPER_ADMIN" },
    {"set_campaign_management": "CAMPAIGN_MANAGEMENT"},
    {"set_issuance_management": "ISSUANCE_MANAGEMENT"}
    ],

    "issuance_management.wasm": 
    [
    {"initialize" : "REGISTRY"},
    {"set_campaign_management": "CAMPAIGN_MANAGEMENT"}
    ],

    "campaign_management.wasm":
    [
        {"initialize": "REGISTRY"},
    {"set_stable_coin_address": "STABLE_COIN" }, 
    ]
 }

contracts = {}
for i in path:
    contract_id = deploy_contract(i, SECRET_KEY)
    name = os.path.basename(i)
    contracts[name] = contract_id
print(contracts)


for contracts_list in function_call.keys():
    for functions in function_call[contracts_list]:
            for func_name in functions:
                print(func_name)
                print(functions[func_name])
                print(contracts_list)
                address = ""
                if functions[func_name] == "SUPER_ADMIN":
                    addr = SUPER_ADMIN
                elif functions[func_name] == "CAMPAIGN_MANAGEMENT":
                    addr = contracts["campaign_management.wasm"]
                elif functions[func_name] == "ISSUANCE_MANAGEMENT":
                    addr = contracts["issuance_management.wasm"] 
                elif functions[func_name] == "REGISTRY":
                    addr = contracts["registry.wasm"] 
                elif functions[func_name] == "STABLE_COIN":
                    addr = STABLE_COIN
                send_tx(contracts[contracts_list], SECRET_KEY, func_name, [scval.to_address(addr)])