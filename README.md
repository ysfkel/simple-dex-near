# Simple Constant Poroduct AMM
## Directories 
- amm - Contains the code for the Automated Market Maker
- tests - contains integration tests
- token - conatains  code for fungible tokens

# AMM Files
- market.core.rs Contains code that defines the core features of the DEX. i.e add_liquidity, remove_liquidity and swap.
- market inspect - Contains code used for viewing contract state
- Market_writer -  Contains code that update the market state
- ft_receivers - Defines receiver for fungible token. It is executed by the transfer_call transfer_call function of the fungible token 


# Testing and Deployments 
 - To run unit tests & integration tests run 
   ```
    npm run test 

