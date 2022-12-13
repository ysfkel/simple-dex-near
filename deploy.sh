# #!/bin/sh

# ./build.sh

# if [ $? -ne 0 ]; then
#   echo ">> Error building contract"
#   exit 1
# fi

# echo ">> Deploying contract"

# # https://docs.near.org/tools/near-cli#near-dev-deploy
# near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/ft_token.wasm

# for run in {1..2}; do
#   echo $1
# done
source .env

case $1 in
    "accounts")
        near create-account app3.$MASTER_ACCOUNT --masterAccount $MASTER_ACCOUNT
        ;;
    "deploy")
        echo "begin deploy.."
        cd contracts/token/
        near dev-deploy --accountId app3.$MASTER_ACCOUNT  --wasmFile ./target/wasm32-unknown-unknown/release/ft_token.wasm --initFunction init_default --initArgs '{"owner_id":"app3.ykel.testnet","total_supply":"1000000000000000000000000000"}'

         ;; 
esac
 