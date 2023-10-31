# near-blank-project

-------------------------------------

  

This app was initialized with [create-near-app]

  
  

# Quick Start

-------------------------------------

  

If you haven't installed dependencies during setup:

  

npm install

  
  

Build and deploy your contract to TestNet with a temporary dev account:

  

npm run deploy

  

Test your contract:

  

npm test

  

If you have a frontend, run `npm start`. This will run a dev server.

  
  

# Exploring The Code  

1. The smart-contract code lives in the `/contract` folder. See the README there for

more info. In blockchain apps the smart contract is the "backend" of your app.

2. The frontend code lives in the `/frontend` folder. `/frontend/index.html` is a great

place to start exploring. Note that it loads in `/frontend/index.js`,

this is your entrypoint to learn how the frontend connects to the NEAR blockchain.

3. Test your contract: `npm test`, this will run the tests in `integration-tests` directory.



# Deploy


Every smart contract in NEAR has its [own associated account][NEAR accounts].

When you run `npm run deploy`, your smart contract gets deployed to the live NEAR TestNet with a temporary dev account.

When you're ready to make it permanent, here's how:

  

## Step 0: Install near-cli (optional)

-------------------------------------

  

[near-cli] is a command line interface (CLI) for interacting with the NEAR blockchain. It was installed to the local `node_modules` folder when you ran `npm install`, but for best ergonomics you may want to install it globally:

  

npm install --global near-cli

  

Or, if you'd rather use the locally-installed version, you can prefix all `near` commands with `npx`

  

Ensure that it's installed with `near --version` (or `npx near --version`)

  
  

## Step 1: Create an account for the contract
  

Each account on NEAR can have at most one contract deployed to it. If you've already created an account such as `your-name.testnet`, you can deploy your contract to `near-blank-project.your-name.testnet`. Assuming you've already created an account on [NEAR Wallet], here's how to create `near-blank-project.your-name.testnet`:

  

1. Authorize NEAR CLI, following the commands it gives you:

  

near login


2. Create a subaccount (replace `YOUR-NAME` below with your actual account name):


near create-account near-blank-project.YOUR-NAME.testnet --masterAccount YOUR-NAME.testnet

  

## Step 2: deploy the contract
  
Use the CLI to deploy the contract to TestNet with your account ID.

Replace `PATH_TO_WASM_FILE` with the `wasm` that was generated in `contract` build directory.

  

near deploy --accountId near-blank-project.YOUR-NAME.testnet --wasmFile PATH_TO_WASM_FILE

  
  

## Step 3: set contract name in your frontend code
  

Modify the line in `src/config.js` that sets the account name of the contract. Set it to the account id you used above.

  

const CONTRACT_NAME = process.env.CONTRACT_NAME || 'near-blank-project.YOUR-NAME.testnet'

  
  
  

# Troubleshooting

On Windows, if you're seeing an error containing `EPERM` it may be related to spaces in your path. Please see [this issue](https://github.com/zkat/npx/issues/209) for more details.

[create-near-app]: https://github.com/near/create-near-app

[Node.js]: https://nodejs.org/en/download/package-manager/

[jest]: https://jestjs.io/

[NEAR accounts]: https://docs.near.org/concepts/basics/account

[NEAR Wallet]: https://wallet.testnet.near.org/

[near-cli]: https://github.com/near/near-cli

[gh-pages]: https://github.com/tschaub/gh-pages

# Use case
## Introduction
The claim feature is an extension of the [Sweat Wallet](https://sweateconomy.com/#) application and aims to safely store the $SWEAT minted for a given users based on their steps provided by the [Sweatcoin Oracle](https://sweatco.in/) and converted to $SWEAT as per the token's [minting curve](https://sweateconomy.com/token). 

Prior to this "claim" feature, $SWEAT accrued from steps was calculated several times per day as determined by the Sweatcoin Oracle and $SWEAT was minted accordingly by the [`token.sweat` ](https://nearblocks.io/address/token.sweat) contract and transferred to the given user's wallet address. The goal of the "claim" feature is to given the user more control over their $SWEAT earned from walking. This is accomplished by diverting minted $SWEAT to a new contract where it will accrue until a user claims it. 

The contract furthermore caters for edge cases in user behaviour which current places the Sweat economy at risk. E.g. If a user churns and disbands the project then there should be a mechanism to recover $SWEAT that was minted to a user's address but abandoned by the user. Currently this is impossible as Sweat Wallet is a self-custody thereby rendering complete control of funds to the user. Having a contract where minted $SWEAT accrues provides a degree of separation in terms of ownerships rights /control of minted $SWEAT. Sweat Wallet may therefore impose a condition that $SWEAT which is not claimed after a set amount of time may be burned from the claim contract. This will not only create a healthier economy (supply vs demand) but furthermore provide a method for maintaining an efficient contract size.

## User Stories
The following users provide some context as to how users will interact with the contract from the Sweat Wallet application. 

### Server side
-------------------------------------

1. As a server I want to impose a 48 hour count down time (offchain) which prevents the "Claim" button from being active. After the 48 hours have matured, the Claim button should be shown as active such that a user may interact with it.
2. As a server I want to inform a user based on internally stored thresholds, when $SWEAT that has accrued has not been claimed after _x_ days and/or _y_ amount. This may be accomplished by push notifications in-app.
3. As a server I want to be alerted when a user has not claimed for 30 days and inform the user that the `burn` method will be invoked in the contract for the server to request to burn accrued $SWEAT that has not been claimed for more than 30 days.
4. As a server I want to regularly monitor contract storage size and evaluate Sweatcoin Oracle data to optimize data entries into the claim contract.
5. As a server I want to limit contract interaction by requiring an active Sweat Wallet UUID that matches a valid keypair to prevent access to the contracts function from independent users (users not using an active Sweat Wallet session)

### Client side
-------------------------------------

1. As a user I want to view my accrued $SWEAT in-app
2. As a user I want to see a progress bar showing how much time is remaining (of the 48 hours) before I can access the Claim button.
3. As a user I want to view claimed $SWEAT in my Available Balance as soon as a successful claim has been initiated.
4. As a user, after claiming, I want to see the countdown timer and associated progress bar reset.
5. As a user I want to be notified when any $SWEAT get burned from my associated associated address in the claim contract.
6. As a user I want to see the gas fee displayed before interacting in any way with the claim contract.
7. As a user I want to receive logical error messages when a contract interaction fails e.g. insufficient NEAR or $SWEAT to cover gas fees, etc. or any reason that causes the contract to panic.
8. As a user I want to receive notifications before an method is called in the claim contract by the server that affects my unclaimed balance in the smart contract (offchain functionality) 
