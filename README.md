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

<img width="818" alt="image" src="https://github.com/Suyog007/Calculator/assets/36821382/4b9a2ee7-99e9-44e8-9aa3-3690a8e239ef">

* Campaign Creation(1):
System owner initiates the creation of a new campaign with specified details. The campaign details include the total amount in stablecoin (e.g. 2,000 USDC) and campaign parameters. 
A smart contract(Campaign Contract), following the Soroban token standard, is deployed on Soroban. The contract mints tokens in an amount equivalent to the stablecoin on deposit (e.g. 2,000 tokens). 

* Token Distribution(2-4): 
Recipients of the campaign are identified and given access to a mobile app or platform. The campaign owner transfers a specific number of tokens (e.g. 2,000 tokens / number of recipients) to each recipient's wallet within the system using the Mobile App. Recipients can use their mobile app to view their token balance. To make a purchase, they visit an authorized merchant. 

* Merchant Interaction(5): 
Merchants have the mobile app integrated with the stellar blockchain platform. Recipients scan a QR code provided by the Merchant and transfer the tokens to the Merchant account. The Merchant, in turn, releases the purchased items to the recipients.

* Fiat Exchange(6): 
After completing the transaction, the merchant can request the campaign owner to exchange their earned tokens for an equivalent amount of fiat currency (e.g., USD). The merchant request burns the tokens and the requested amount of stablecoin tokens from the campaign management contract is transferred to super admin. For the MVP the exchange to fiat will take place with the merchant in person.

* Mobile dApps:
The stellar wallet is integrated on this app and it handles every transaction for the owner/campaign creator, Merchant or Recipients.

# Smart Contracts Specification

<img width="1094" alt="image" src="https://github.com/Suyog007/Calculator/assets/36821382/3c352ac6-489b-4df5-a46d-dea6119bb50c">

* Token Contract:
The Super Admin possesses the authority to generate new tokens. When creating a campaign, the campaign creator must specify the token they intend to use for the campaign. Each token will be linked to its unique roster of merchants and items. For the MVP there will only be one token contract, but in later iterations of LocalCoin it will be possible to deploy different token contracts that have different approved vendor and items lists. If a campaign creator is satisfied with the approved merchant and items list defined for a given token they will not need to go through the process of defining and deploying a new token.

* Issuance Management:
The Issuance Management contract will maintain the definitions of each token contract that gets deployed. This includes the approved merchants list and the approved items list. When a campaign creator creates a new campaign and chooses a token to use, a new Campaign contract will be deployed, for which the owner may enable additional admin addresses.

* Campaign Management:
The Campaign Management contract oversees all campaigns, requiring the sender to submit stable coins upon campaign creation. All stable coins are held by the Campaign Management contract. This same contract is responsible for deploying a new campaign contract. Upon the settlement of funds from merchants, the Campaign Management contract will execute the burning of tokens.

* User Registry
The user registry is a place for approved merchants, campaign owners and admins. It is not expected that there will be any need to track recipients. In fact, privacy preserving transactions for recipients is on the LocalCoin future feature list. Zk technology is not quite there yet in terms of usability for a product like this.

* Campaign Contract
Campaign Contract is a contract that holds the tokens worth of the stable coin. The campaign creator needs to interact with this contract to send the token to the recipients. For a given token, spending restrictions will be defined limiting transfers to approved merchants for approved items. For the MVP, only the merchant restrictions will be enforced on-chain. The merchant will be trusted to limit purchases to approved items.

We've created a detailed diagram showing all the agreements within our product. 

<img width="1094" alt="image" src="https://github.com/Suyog007/Calculator/assets/36821382/3c83bade-8d42-4180-8a91-b99e694a934a">


# CampaignCreator Flow API
* Create Campaign:
Campaign Creator can create a campaign by calling Campaign Management Contract:

    > create_campaign(name:String, description:String,
        no_of_recipients:u32, token_address:Address,
        amount:i128, creator: Address)

    >name: Name of the campaign,<br />
    description: Description of the campaign,<br />
    no_of_recipients: total number of recipients of the campaign,<br />
    token_address: token address that is to be used on that campaign,<br />
    amount: Total amount of stable coin to be transferred,
    creator: Address of Campaign Creator

* Check Ongoing Campaign:
To view all the campaigns Campaign Management Contract needs to be called.


    >get_campaigns(creator: Address)</br>
    creator: Address of Creator whose campaigns is to be viewed

    The response is:
    >[ campaignAddress1, CampaignAddress2, ...]

* View Campaign:
To view all the campaigns of the specific campaign creator, Campaign Management Contract needs to be called.


    >get_creator_campaigns(creator: Address)
    </br>creator: Address of Creator whose campaigns is to be viewed

    The response is:
    >[ {campaign:GCHOM….. , token:GCHOM….., token_minted:100, info: ["first_campaign", "description_of_campaign", "no of recipients"] }, {campaign:GCHOM….. , token:GCHOM….., token_minted:200, info: ["second_campaign", "description_of_campaign", "no of recipients"] }, ….]

* Get Campaign Details
    To view the campaign details, Campaign Contract needs to be called.


    >get_campaign_info(campaign: Address)
    </br>campaign: Address of Campaign whose details is to be viewed

    The response is:
    >["name_of_the_campaign", "description_of_the_campaign", "no of recipients"]

* Transfer Funds to Recipients
To transfer the funds from campaign creator to recipients, creator needs to call Campaign Contract.

    >transfer_tokens_to_recipient(to:Address, amount:i128)
    </br>to: Address of Recipient,</br>
    amount: amount to be transferred

# Recipient Flow

* Transfer Funds to Merchant
    To transfer the funds from recipients to merchants, users need to call the LocalCoin Token Address that they are transferring.


    >recipient_to_merchant_transfer(from: Address, to: Address, amount: i128)
    from: Address of Recipient</br>
    to: Address of Merchant,</br>
    amount: amount to be transferred</br>

* View Tokens
    To view all the tokens that the recipient has, the user needs to call Issuance Management Contract.


    >get_balance_of_batch(user:Address)
    </br>user: Address of the entity whose token balance is to be viewed

    The response is:
    >{"token1": 100, "token2":200, ..}

# Merchant Flow

* To Register for Merchant
    To apply for the registration of merchants, merchant needs to call Issuance Management Contract:


    > merchant_registration(merchant_wallet:Address, proprietor:String, phone_no:String, store_name:String, location:String)
    </br>merchant_wallet: Address of Merchant
    </br>proprietor: Proprietor,
    </br>phone_no: Phone_no,
    </br>store_name: Store_name,
    </br>location: Location

* Request Settlement
    To request the settlement with the super admin, the merchant needs to call Campaign Management Contract.

    >request_campaign_settelment(from:Address, amount:i128, token_address:Address)
    </br>from: Address of Merchant
    </br>token_address: Address of the token that is to be settled,
    </br>amount: amount to be settled
