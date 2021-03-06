# oc-governance-bot

This bot runs entirely on the Internet Computer. It listens for open NNS proposals, using the IC heartbeat functionality to constantly poll the NNS governance canister. When an open proposal is found, its information is put into a text message that is then sent to the corresponding OpenChat group for that proposal's topic.

OpenChat currently isn't open source, but the team shared enough Candid information with me that I was able to get my bot to work. If you want to create your own bot, below I show what it took to get my bot going.

1. Register the bot as a user on OpenChat
2. Create the bot's user canister on OpenChat
3. Join the bot to a group
4. Test sending a message

### Register the bot as a user on OpenChat

Simply call [register_governance_bot_user](https://github.com/lastmjs/oc-governance-bot/blob/main/canisters/bot/src/bot_utilities.rs#L25) with the correct username. You will need to modify this function with your bot's username.

### Create the bot's user canister on OpenChat

Simply call [create_governance_bot_user_canister](https://github.com/lastmjs/oc-governance-bot/blob/main/canisters/bot/src/bot_utilities.rs#L38). This will instruct OpenChat to register the canister of this user.

### Join the bot to a group

Hopefully you have the bot user's canister id from the previous step. If not you can call [get_governance_bot_user_canister_id](https://github.com/lastmjs/oc-governance-bot/blob/main/canisters/bot/src/bot_utilities.rs#L49) to get it, or you can join a chat with the bot in the OpenChat UI and look at the canister id in the address bar of the browser.

Next you will call [governance_bot_join_group](https://github.com/lastmjs/oc-governance-bot/blob/main/canisters/bot/src/bot_utilities.rs#L70) and pass in the canister id of the group that you want to join. You should have created the group already. The canister id can be found in the address bar of the browser.

### Test sending a message

You can test your bot using the [send_message](https://github.com/lastmjs/oc-governance-bot/blob/main/canisters/bot/src/bot_utilities.rs#L84) function. You might need to modify it to update the message id (if you have already used `0` as a message id) and the bot's username.

## Running the bot

The key to running the bot is the [heartbeat](https://github.com/lastmjs/oc-governance-bot/blob/main/canisters/bot/src/lib.rs#L52) function. Once you push that to the IC, the `heartbeat` function will run every few seconds. You can read more about `canister_heartbeat` here: https://smartcontracts.org/docs/interface-spec/index.html#_heartbeat

The logic should be pretty easy to follow if you start at the `heartbeat` function. Basically every few seconds the NNS governance canister is polled to see if there are any open proposals. As long as the proposal id has not already been processed (stored locally in a map), then a message is sent to the appropriate group.