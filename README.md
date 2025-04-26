# SteamPolice

> [!WARNING]
> **I am not responsible for any bans or other consequences that may arise from using this program.**
> **Use at your own risk.**

## to use you need

-   some brain
-   cup of tea
-   internet connection
-   10+ iq (optional)
-   cargo run

# instalation

## 1. install rust (https://www.rust-lang.org/tools/install)

## 2. run program

```bash
cargo run --release
```

## program will create 2 files in the same directory as the executable

### accounts.json:

> example schema

```json
[
    {
        "name": "foo bar",
        "session_id": "bruh",
        "steam_login_secure": "bla bla ble"
    }
]
```

### filters.json:

> example schema

```json
["example_filter", "example_filter2", "example_filter3"]
```

> it will work like this: If comment text contains any of the filters it will be report it (or something else)

# USAGE

## program will ask you what you want to do

### 1. report comments on user profile

### 2. report profiles (reasons are customizable in words.json)

### 3. post comments on user profile

## all of these actions can be done with user id or profile link (also support profile searching)

### Examples:

#### user id, profile link:

```
🎯 What action do you want to perform?
1. Report comments on profiles
2. Report profiles
3. Post comments on profiles
➡️  Enter your choice (1, 2, 3): 1
🎯 Enter target profile ID(s) / Nickname(s) / Link(s) (comma-separated, use 'search:term!>limit' for search): dest4590
🔗 Resolved 'dest4590' to profile ID: 1337
🎯 Targeting 1 profile(s): [1337]
⚙️  Enter keywords to filter comments (comma-separated, or press Enter to use autofilters):
📁  Loading filters from 'filters.json'...
✅  Loaded 14 filter(s) from file.
```

> program will automatically report all comments on the profile that equals any of the filters from filters.json file

#### user search (require STEAM API key):

```
🎯 Enter target profile ID(s) / Nickname(s) / Link(s) (comma-separated, use 'search:term!>limit' for search): search:profile!>5
🔍  Searching for profiles matching: 'profile'
🌐  Fetching search results page 1...
```

> program will search for profiles matching the term "profile" and limit the results to 5 profiles. If it finds any, it will report all comments on those profiles that match any of the filters from filters.json file.
