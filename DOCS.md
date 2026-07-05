# Bounty Bot

Documentation for the Bounty Bot developed for the [Fluxer Wild West](https://fluxer.gg/NiJD7OIT) community.

## Summary

**Bounty Bot** functions similarly to "FluxBug" but manages bounties instead of bug reports. **Bounty Creators** submit bounties using a template, which Bounty Bot validates and posts to the `#Approval Queue` channel. After approval from both **Fluxer Staff** and **Bounty Managers**, the bounty appears in the `#bounties` channel for **Bounty Hunters** to claim. **Stakeholders** can contribute additional funds to specific bounties. Once completed, **Bounty Managers** finalize the bounty and process payment to the assigned hunter.

> [!Important]
> Payment handling is managed separately and falls outside this bot's scope.
> Contact Bounty Managers for payment-related questions.

## What is a Bounty?

Bounty is a **paid task linked to a project issue**. All bounties must receive approval from both Fluxer Staff and Bounty Managers before becoming available to hunters.

**Each bounty contains:**

- **Bounty ID** - Unique Identifier used across the community
- **Creator** - Bounty Creator who submitted the bounty
- **Content** - Title, Due Date, Issue URL, Additional Information, Judging Criteria, and Bounty Amount
- **Status** - Bounty's current stage (Pending, Approved, Assigned, Completed, or Rejected)
- **Creation Date** - When the bounty was originally submitted

## What is a Stakeholder?

A **stakeholder** is a community member who contributes funds to a bounty.

## What is a Bounty Creator?

A **bounty creator** is a community member who submits a new bounty for an issue.

## What is a Bounty Hunter?

A **bounty hunter** is a community member who completes bounties. Hunters can self-assign approved bounties to work on them.

## Permissions

Commands require specific roles and permissions. Community Administrators always have all permissions.

| Command Syntax                          | Description                                 | Aliases                                   |
| --------------------------------------- | ------------------------------------------- | ----------------------------------------- |
| `add-permission <role> <permission>`    | Add a bounty bot permission to a role.      | None                                      |
| `remove-permission <role> <permission>` | Remove a bounty bot permission from a role. | `rm-permission`                           |
| `permissions`                           | List all permissions in the community.      | `perms`, `list-perms`, `list-permissions` |

### Parameters

`permission` can be one of:

- `manage-community-config`
- `manage-bounties`
- `create-bounty`
- `bounty-hunter`

> [!Note]
> Other permission name spellings are supported but not documented here.
> See `commands::permission_management::parse_permission_str` for the complete list.

`role` can be:

- A role @mention (e.g., `@Bounty Managers`)
- A role ID

## Commands

The following is a complete command reference. Command syntax follows the [Minecraft Wiki](https://minecraft.wiki/w/Commands#Syntax) convention. The default prefix is `b!`.

## Community Management

> [!Important]
> Requires Community **Administrator** permission or `MANAGE_GUILD_CONFIG` permission.

| Command Syntax                               | Description                                                |
| -------------------------------------------- | ---------------------------------------------------------- |
| `config`                                     | Display the current community configuration.               |
| `config <channel-type> [<channel> \| reset]` | Set or reset a channel for a specific purpose.             |
| `config prefix <new_prefix>`                 | Set the command prefix.                                    |
| `config delete-commands <true \| false>`     | Automatically delete commands and replies after 5 seconds. |

**Aliases:** `communityconfig`, `community-config`, `guildconfig`, `guild-config`, `serverconfig`, `server-config`, `cfg`

### Channel Types

Use these values for `<channel-type>`:

- `bounty-submission-channel` - Where bounty creators submit new bounties.
- `approval-queue-channel` - Where submitted bounties await review.
- `approved-bounties-channel` - Where approved bounties are displayed.
- `assigned-bounties-channel` - Where assigned bounties are displayed.
- `completed-bounties-channel` - Where completed bounties are archived.
- `rejected-bounties-channel` - Where rejected bounties are archived.

## Bounty Management

| Command Syntax                                       | Role           | Description                                                                                               |
| ---------------------------------------------------- | -------------- | --------------------------------------------------------------------------------------------------------- |
| `self-assign <bounty_id>`                            | Bounty Hunter  | Assign an approved bounty to yourself (adding you to the queue if someone else was already assigned).     |
| `self-unassign <bounty_id>`                          | Bounty Hunter  | Unassign yourself from a bounty (you may lose your spot in the queue if you were queued).                 |
| `assign <bounty_id> <user>`                          | Bounty Manager | Assign a bounty to a community member (may add the member to the queue instead).                          |
| `unassign <bounty_id> <user>`                        | Bounty Manager | Unassign the user from a bounty or remove them from the queue.                                            |
| `approve <bounty_id>`                                | Bounty Manager | Approve a pending bounty.                                                                                 |
| `complete <bounty_id>`                               | Bounty Manager | Mark a bounty as completed.                                                                               |
| `edit <bounty_id> <field> [value]`                   | Bounty Manager | Edit a bounty field.                                                                                      |
| `reject <bounty_id>`                                 | Bounty Manager | Reject a bounty.                                                                                          |
| `delete <bounty_id>`                                 | Bounty Manager | **Permanently delete a bounty (cannot be undone).**                                                       |
| `stakeholder add <bounty_id> <amount> <user> [note]` | Bounty Manager | Add a stakeholder contribution.                                                                           |
| `stakeholder remove <bounty_id> <user>`              | Bounty Manager | **Remove all stakeholder contributions (cannot be undone).**                                              |

### Bounty Management Aliases

- `self-assign`: `selfassign`
- `self-unassign`: `selfunassign`
- `assign`: `assign-to`, `assign-to-bounty`, `bounty-assign`
- `unassign`: `unassign-from`, `unassign-from-bounty`, `bounty-unassign`
- `approve`: `approve-bounty`, `bounty-approve`
- `complete`: `complete-bounty`, `bounty-complete`
- `reject`: `deny`, `reject-bounty`, `bounty-reject`, `deny-bounty`, `bounty-deny`
- `delete`: `delete-bounty`, `deletebounty`, `removebounty`, `bounty-delete`, `bountydel`, `bountyrm`, `rmbounty`, `delbounty`
- `stakeholder remove`: `stakeholder rm`
- `edit`: `edit-bounty`

### Field Details

- `stakeholder add` - Add a stakeholder contribution to a bounty. A community member can have multiple contributions. Amount format: number, `$` prefix, or `ct` suffix (USD default).
- `edit` - Edit a bounty. `field` can be `title`, `additional-info`, `proposed-amount`, `due-date`, `issue-url`, or `judging-criteria`. Omit value to remove the field.

## Misc commands

No permissions required.

| Command Syntax    | Description                                 |
| ----------------- | ------------------------------------------- |
| `bounty-workflow` | Display a flowchart of the bounty workflow. |
| `ping`            | Pong!                                       |

**Aliases:** `bountyworkflow`, `workflow`, `bounty-workflow-image`
