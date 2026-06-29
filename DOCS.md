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

| Command syntax                          | Description                                 | Aliases                                   |
| --------------------------------------- | ------------------------------------------- | ----------------------------------------- |
| `add-permission <role> <permission>`    | Add a bounty bot permission to a role.      | _none_                                    |
| `remove-permission <role> <permission>` | Remove a bounty bot permission from a role. | `rm-permission`                           |
| `permissions`                           | List the community's permissions.           | `perms`, `list-perms`, `list-permissions` |

`permission` can be one of `create-bounty`, `manage-bounties`, `manage-community-config`, `bounty-hunter` (other spellings are supported but that would be too much to document. For a real full list of the permission names, see `commands::permission_management::parse_permission_str`)

`role` can be a role @mention, a role ID, `@everyone` or `everyone`.

## Commands

The following is a complete command reference. Command syntax follows the [Minecraft Wiki convention](https://minecraft.wiki/w/Commands#Syntax). The default prefix is `b!`.

### Community Management

> [!Important]
> Requires **Administrator** permission. Or `MANAGE_GUILD_CONFIG` permission.

| Command Syntax                                           | Description                                                                                                                    |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------- | ------- | ----------------------------------------------------------------------------------- |
| `config`                                                 | Display the current community configuration.                                                                                   |
| `config bounty-submission-channel [<channel> \| reset]`  | Set or reset the channel where bounty creators submit new bounties.                                                            |
| `config approval-queue-channel [<channel> \| reset]`     | Set or reset the approval queue channel where submitted bounties await review.                                                 |
| `config approved-bounties-channel [<channel> \| reset]`  | Set or reset the channel where approved bounties are displayed.                                                                |
| `config (claimed-bounties-channel                        | assigned-bounties-channel) [<channel> \| reset]`                                                                               | Set or reset the channel where assigned bounties are displayed. |
| `config completed-bounties-channel [<channel> \| reset]` | Set or reset the channel where completed bounties are archived.                                                                |
| `config rejected-bounties-channel [<channel> \| reset]`  | Set or reset the channel where rejected bounties are archived.                                                                 |
| `config prefix <new_prefix>`                             | Set the command prefix (default: `b!`). Independent of the current prefix, you can always run commands by @mentioning the bot. |
| `config delete-commands (yes                             | true                                                                                                                           | no                                                              | false)` | Whether commands and their replies should be automatically deleted after 5 seconds. |

_**Aliases for `config`:** `communityconfig`, `community-config`, `guildconfig`, `guild-config`, `serverconfig`, `server-config`, `cfg`_

### Bounty Management

Permissions are specified for each command.

| Command Syntax                                       | Role           | Description                                                                                                                                                                                                                  | Aliases                                                                                                            |
| ---------------------------------------------------- | -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `self-assign <bounty_id>`                            | Bounty Hunter  | Assign an approved bounty to yourself. Only works if the bounty is unassigned.                                                                                                                                               | `selfassign`                                                                                                       |
| `self-unassign <bounty_id>`                          | Bounty Hunter  | Unassign yourself from a bounty.                                                                                                                                                                                             | `selfunassign`                                                                                                     |
| `assign <bounty_id> <user>`                          | Bounty Manager | Assign a bounty to a specific community member, even if already assigned. _(Unassigning not yet supported)_                                                                                                                  | `assign-to`, `assign-to-bounty`, `bounty-assign`                                                                   |
| `unassign <bounty_id>`                               | Bounty Manager | Unassign whoever is assigned to the specified bouny.                                                                                                                                                                         | `unassign-from`, `unassign-from-bounty`, `bounty-unassign`                                                         |
| `approve <bounty_id> [comment]`                      | Bounty Manager | Record an approval vote on a pending bounty. The bounty moves to the approved bounties channel once it has 3 approvals. Comments are shown on the bounty embed.                                                              | `approve-bounty`, `bounty-approve`                                                                                 |
| `approve-bypass <bounty_id> [comment]`               | Bounty Manager | Immediately approve a pending bounty, bypassing the approval threshold. The bypass and optional comment are shown on the bounty embed.                                                                                        | `approval-bypass`, `bypass-approve`, `bounty-approve-bypass`                                                       |
| `complete <bounty_id>`                               | Bounty Manager | Mark a bounty as completed and move it to the completed bounties channel.                                                                                                                                                    | `complete-bounty`, `bounty-complete`                                                                               |
| `reject <bounty_id> [comment]`                       | Bounty Manager | Record a denial vote on a pending bounty. The bounty moves to the rejected bounties channel once it has 2 denials. Comments are shown on the bounty embed.                                                                    | `deny`, `reject-bounty`, `bounty-reject`, `deny-bounty`, `bounty-deny`                                             |
| `reject-bypass <bounty_id> [comment]`                | Bounty Manager | Immediately reject a pending bounty, bypassing the denial threshold. The bypass and optional comment are shown on the bounty embed.                                                                                           | `deny-bypass`, `denial-bypass`, `bypass-reject`, `bypass-deny`, `bounty-deny-bypass`                              |
| `delete <bounty_id>`                                 | Bounty Manager | ‼️ Permanently delete a bounty. **(Cannot be undone)**                                                                                                                                                                       | `delete-bounty`, `deletebounty`, `removebounty`, `bounty-delete`, `bountydel`, `bountyrm`, `rmbounty`, `delbounty` |
| `stakeholder add <bounty_id> <amount> <user> [note]` | Bounty Manager | Add a stakeholder contribution to a bounty. A community member can have multiple contributions. Format: amount as a number, `$` prefix, or `ct` suffix _(amounts default to USD)_.                                           |                                                                                                                    |
| `stakeholder remove <bounty_id> <user>`              | Bounty Manager | ‼️ Remove all stakeholder contributions from a community member on a bounty. **(Cannot be undone)**                                                                                                                          | `stakeholder rm`                                                                                                   |
| `edit <bounty_id> <field> [value]`                   | Bounty Manager | Edit a bounty. `field` can be one of `title`, `additional-info`, `proposed-amount`, `due-date`, `issue-url` or `judging_criteria`. If the `value` is not specified, will remove the specified field from the bounty content. | `edit-bounty`                                                                                                      |

### Misc commands

No permissions required.

| Command Syntax    | Description                                 |
| ----------------- | ------------------------------------------- |
| `bounty-workflow` | Display a flowchart of the bounty workflow. |
| `ping`            | Pong!                                       |

**Aliases for `bounty-workflow`:** `bountyworkflow`, `workflow`, `bounty-workflow-image`
