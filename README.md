# Localcoin
LocalCoin is a closed cycle token system designed to enable on-chain control and tracking of financial incentives in the real world. It is a product and service that manages the disbursement of funds which have restrictions on how they may be spent, comprising smart contracts on Soroban, a web app and mobile app, and eventually a payment card and seamless DAO integration.

The LocalCoin platform solves problems faced by philanthropic, or social good organizations regarding trust and transparency of how funds are used. The problems include:

* A lack of transparency for donors into how their donations are used.
* Donor resistance due to the lack of transparency, which limits trust.
* A risk of funds being misused if cash assistance is distributed.
* Possibility that unrestricted funds may quickly leave the community, like going to Amazon purchases, limiting their impact.

The LocalCoin platform addresses these problems by:

* Providing a closed loop system
* Providing a mobile dApp that can be used by both merchants and participants to exchange the local currency for the purchase of approved items.
* Enabling transactions with the local currency to be traced by recording them all on chain.
* Allowing definition of flexibly defined restrictions for how the local currency may be spent, with a list of approved merchants and approved items for purchase.
* Allowing for different issuances of currency to have different restrictions.
* Limiting transfers only to approved merchants will make trading outside the system infeasible.
* Merchants will be able to redeem the local currency for USD on demand.

The intended audience for the LocalCoin platform is:

* Local, social good DAOs - This seems like the ideal audience, for which LocalCoin can satisfy the need for fully on-chain governance and execution.
* Stand alone philanthropic organizations
* NGOs operating in developing countries


# Product
A local currency system built on the Stellar Soroban platform that creates tokens with custom
spending restrictions which can be issued by the campaign owner (backed by an equivalent
amount of stablecoin) and spent by the recipients with authorized merchants. Merchants can
finally exchange the tokens with the campaign owner for the local fiat currency thus completing
the loop.

# System Architecture
A simplified architecture will be built for the MVP, roughly shown in the diagram below.

<img width="647" alt="system_architecture_localcoin" src="https://github.com/venture-23/localcoin/assets/60603625/eae041e2-fa09-48bd-96a1-df50b6f5ff0f">

* Campaign Creation(1):
System owner initiates the creation of a new campaign with specified details. The campaign details include the total amount in stablecoin (e.g. 2,000 USDC) and campaign parameters. 
A smart contract, following the Soroban token standard, is deployed on Soroban, if required. The contract mints tokens in an amount equivalent to the stablecoin on deposit (e.g. 2,000 tokens). 

* Token Distribution(2-4): 
Recipients of the campaign are identified and given access to a mobile app or platform. The campaign owner transfers a specific number of tokens (e.g. 2,000 tokens / number of recipients) to each recipient's wallet within the system using the Mobile App. Recipients can use their mobile app to view their token balance. To make a purchase, they visit an authorized merchant. 

* Merchant Interaction(5): 
Merchants have the mobile app integrated with the stellar blockchain platform. Recipients scan a QR code provided by the Merchant and transfer the tokens to the Merchant account. The Merchant, in turn, releases the purchased items to the recipients.

* Fiat Exchange(6): 
After completing the transaction, the merchant can request the campaign owner to exchange their earned tokens for an equivalent amount of fiat currency (e.g., USD). The merchant request sends the tokens to an escrow contract. The campaign owner initiates an exchange by sending the requested amount of stablecoin tokens from the campaign management contract for conversion and deposit into the merchant's bank account. The token is then burned from the escrow contract once the currency is exchanged. For the MVP the exchange to fiat will take place with the merchant in person, and the tokens will be sent to a burn address on the campaign management contract when the merchant makes their redemption request.

* Mobile dApps:
The stellar wallet is integrated on this app and it handles every transaction for the owner/campaign creator, Merchant or Recipients.










 

