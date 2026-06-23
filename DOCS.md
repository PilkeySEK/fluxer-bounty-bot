# Bounty bot
Documentation for the bounty bot developed for the "Fluxer Wild West" community.

Where "FWW" = "Fluxer Wild West"

## Summary
The bot shares major similarities with the "FluxBug" bot in the "Desktop Canary Testers" and "Fluxer Mobile" official communities, but instead of bug reports it is used for dev bounties. There is a template for submitting bounties which is sent in a channel. If the template is correct, the bounty bot will then post it in the channel for pending bounties. New bounties need to be approved by FWW Staff before being sent in the channel for unclaimed bounties. Any bounty hunter can self-assign a bounty. Stakeholders can request to be added with a specified sum of money (the details of how the money is transferred and handled are not in scope for this bot, at least currently). Once completed (e.g. merged into Fluxer), FWW Staff mark the bounty as completed (and it will be moved to the channel for completed bounties) and the bounty hunter should be paid.

## What is a bounty, exactly?
Each bounty has the following information attached to it:
* The bounty ID which is unique across the whole community
* Who originally submitted the bounty
* The content: Title, Due Date (Deadline), Issue URL, Additional Info, Judging Criteria and Requested Amount (Pay). Some of this information may be optional.
* The state: Completed, Approved, Pending or Rejected
* When the bounty was created

## What is a stakeholder?
A "stakeholder" is someone who pays money for the completion of a bounty.

## What is a bounty hunter?
A bounty hunter is someone who is allowed to self-assign bounties for themselves to work on. They need to be approved by FWW Staff first.

## Permissions
For many commands, the user needs certain permissions. Currently, there is no way to configure them but this will be added very soon. FWW Administrators (in Fluxer) always have all permissions.

## Commands
Now follows a complete list of all commands. The notation used to specify command syntax is the same as the one used in the minecraft wiki: https://minecraft.wiki/w/Commands#Syntax

The prefix will be omitted (set to `b!` by default, currently there is no way to change it).

### Community management
These commands require the "Manage community configuration" permission.

`config bounty-submission-channel [<channel> | reset]` - Set or reset the channel where bounty submissions should be sent by people to create new bounties.

`config approval-queue-channel [<channel> | reset]` - Set or reset the channel where bounties will be sent after being submitted, the approval queue.

`config approved-bounties-channel [<channel> | reset]` - Set or reset the channel where bounties will be sent after being approved.

`config claimed-bounties-channel [<channel> | reset]` - Set or reset the channel where bounties that have been claimed by a bounty hunter will be sent.

`config completed-bounties-channel [<channel> | reset]` - Set or reset the channel where completed bounties will be sent.

`config (rejected-bounties-channel | denied-bounties-channel) [<channel> | reset]` - Set or reset the channel where rejected bounties will be sent.

`config` - The bot will reply with the current community configuration.

**Alises of `config`:** `communityconfig`, `community-config`, `guildconfig`, `guild-config`, `serverconfig`, `server-config`, `cfg` 

### Misc commands
These commands don't require any permissions.

`ping` - Pong!

`bounty-workflow` - Replies with a flowchart of the current workflow for a bounty. **Aliases:** `bountyworkflow`, `workflow`, `bounty-workflow-image`

### Bounty commands
Permissions are specified for each command.

`complete <bounty id>` (Manage Bounties) - Mark a bounty as completed and move it to the channel for completed bounties. **Aliases:** `complete-bounty`, `bounty-complete`

`approve <bounty id>` (Manage Bounties) - Approve a bounty and move it to the channel for approved bounties. **Aliases:** `approve-bounty`, `bounty-approve`

`reject <bounty id>` (Manage Bounties) - Reject/Deny a bounty and move it to the channel for rejected bounties. **Aliases:** `deny`, `reject-bounty`, `bounty-reject`, `deny-bounty`, `bounty-deny`

`delete <bounty id>` (Manage Bounties) - Delete a bounty fully. Will delete the message for the bounty in the channel where it is currently in. This cannot be undone. **Aliases:** `delete-bounty`, `deletebounty`,  `removebounty`, `bounty-delete`, `bountydel`, `bountyrm`, `rmbounty`, `delbounty`

`assign <bounty id> <user>` (Manage Bounties) - Assign a bounty to a specific user, even if someone is already assigned to that bounty. Currently, users cannot be unassigned. **Aliases:** `assign-to`, `assign-to-bounty`, `bounty-assign`

`self-assign <bounty id>` (Bounty Hunter) - Assign a bounty to yourself. Only allowed if the bounty is in the "approved" state. Only possible when someone is not already assigned to the bounty. Currently, someone cannot unassign themselves from a bounty. **Aliases:** `selfassign`

`stakeholder add <bounty id> <amount> <user> [note]` (Manage Bounties) - Add a stakeholder to the bounty. Note that one user can have multiple "stakeholder positions". `amount` can be either just a number, start with a `$` or end with `ct`. If `amount` is just a number without a dollar or cent sign, it will be interpreted as dollars.

`stakeholder (remove | rm) <bounty id> <user>` (Manage Bounties) - Remove a stakeholder from a bounty. This will remove all of the stakeholder's "stakeholder positions" from the bounty. Cannot be undone.

## TODOs
* Unassigning people and self from bounties
* Setting the command prefix
* Managing permissions (linked to roles or users)
* Editing bounties