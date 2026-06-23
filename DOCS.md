# Bounty Bot
Documentation for the Bounty Bot developed for the Fluxer Wild West (FWW) community.

> [!Note]
>
> "FWW" stands for "Fluxer Wild West"

## Summary
Bounty Bot shares major similarities with the "FluxBug" bot in the "Desktop Canary Testers" and "Fluxer Mobile" official communities, but instead of bug reports it is used for bounties. There is a template for submitting bounties which is sent in a channel. If the template is correct, the bounty bot will then post it in the channel for approval queue bounties. New bounties need to be approved by Fluxer Staff and FWW Staff before being sent in the channel for unclaimed bounties. Any bounty hunter can self-assign a bounty. Stakeholders can request to be added with a specified sum of money. Once completed (e.g. merged into Fluxer repository), FWW Staff mark the bounty as completed (and it will be moved to the channel for completed bounties), and the assigned bounty hunter gets paid.

> [!Important]
>
> Payment handling is managed separately and is outside this bot's scope.

## What is a Bounty, exactly?
A bounty is a paid task linked to a project issue. All bounties require approval from both Fluxer Staff and FWW Staff before being listed for hunters.

**Each bounty includes:**
- **Bounty ID:** Unique ID across the community
- **Creator:** Community member who created or submitted the bounty
- **Content:** Title, Due Date, Issue URL, Additional Information, Judging Criteria, and Bounty Amount
- **Status:** Pending, Approved, Assigned, Completed, or Rejected
- **Creation date:** When the bounty was submitted

## What is a Stakeholder?
A "stakeholder" is a community member who contributes funds to a bounty.

## What is a Bounty Creator?
A "bounty creator" is a community member who submits a new bounty for an issue.

## What is a Bounty Hunter?
A "bounty hunter" is a community member who completes bounties. Hunters can self-assign approved bounties to work on them.

## Permissions
Commands require specific roles and permissions. FWW Administrators always have all permissions. 

> [!Note]
>
> Permission configuration is coming soon.

## Commands
The following is a complete command reference. Command syntax follows the [Minecraft Wiki convention](https://minecraft.wiki/w/Commands#Syntax). The default prefix is `b!` (currently non-configurable).

### Community Management

> [!Important]
> Requires **Administrator** permission.

| Command Syntax | Description |
| --- | --- |
| `config` | Display the current community configuration. |
| `config bounty-submission-channel [<channel> \| reset]` | Set or reset the channel where bounty creators submit new bounties. |
| `config approval-queue-channel [<channel> \| reset]` | Set or reset the approval queue channel where submitted bounties await review. |
| `config approved-bounties-channel [<channel> \| reset]` | Set or reset the channel where approved bounties are displayed. |
| `config assigned-bounties-channel [<channel> \| reset]` | Set or reset the channel where assigned bounties are displayed. |
| `config completed-bounties-channel [<channel> \| reset]` | Set or reset the channel where completed bounties are archived. |
| `config rejected-bounties-channel [<channel> \| reset]` | Set or reset the channel where rejected bounties are archived. |

_**Aliases for `config`:** `communityconfig`, `community-config`, `guildconfig`, `guild-config`, `serverconfig`, `server-config`, `cfg`_

### Bounty Management
Permissions are specified for each command.

| Command Syntax | Role | Description | Aliases |
| --- | --- | --- | --- |
| `self-assign <bounty_id>` | Bounty Hunter | Assign an approved bounty to yourself. Only works if the bounty is unassigned. _(Unassigning not yet supported)_ | `selfassign` |
| `assign <bounty_id> <user>` | Bounty Manager | Assign a bounty to a specific community member, even if already assigned. _(Unassigning not yet supported)_ | `assign-to`, `assign-to-bounty`, `bounty-assign` |
| `approve <bounty_id>` | Bounty Manager | Approve a pending bounty and move it to the approved bounties channel. | `approve-bounty`, `bounty-approve` |
| `complete <bounty_id>` | Bounty Manager | Mark a bounty as completed and move it to the completed bounties channel. | `complete-bounty`, `bounty-complete` |
| `reject <bounty_id>` | Bounty Manager | Reject a bounty and move it to the rejected bounties channel. | `deny`, `reject-bounty`, `bounty-reject`, `deny-bounty`, `bounty-deny` |
| `delete <bounty_id>` | Bounty Manager | ‼️ Permanently delete a bounty. **(Cannot be undone)** | `delete-bounty`, `deletebounty`,  `removebounty`, `bounty-delete`, `bountydel`, `bountyrm`, `rmbounty`, `delbounty` |
| `stakeholder add <bounty_id> <amount> <user> [note]` | Bounty Manager | Add a stakeholder contribution to a bounty. A community member can have multiple contributions. Format: amount as a number, `$` prefix, or `ct` suffix _(amounts default to USD)_. | | 
| `stakeholder remove <bounty_id> <user>` | Bounty Manager | ‼️ Remove all stakeholder contributions from a community member on a bounty. **(Cannot be undone)** | `stakeholder rm` |

### Misc commands
No permissions required.

| Command Syntax | Description |
| --- | --- |
| `bounty-workflow` | Display a flowchart of the bounty workflow. |
| `ping` | Pong! |

**Aliases for `bounty-workflow`:** `bountyworkflow`, `workflow`, `bounty-workflow-image`

## TODOs
- [ ] Unassign bounties (for bounty hunters and managers)
- [ ] Configurable command prefix
- [ ] Role-based permission management
- [ ] Edit existing bounties
- [ ] Fix duplicate due date
