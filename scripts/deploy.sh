#!/bin/sh

source dev.env

make build

if [ $? -ne 0 ]; then
  echo ">> Error building contract"
  exit 1
fi

echo ">> Deploying contract"

near dev-deploy --wasmFile "res/sweat_claim.wasm" --initFunction "init" --initArgs "{\"token_account_id\": \"$TOKEN_ACCOUNT_ID\", \"manager\": \"$ADMIN_ACCOUNT_ID\", \"fee_account_id\": \"$FEE_ACCOUNT_ID\"}"
