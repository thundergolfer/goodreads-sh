<h1 align="center">goodreads-sh</h1>
<p align="center"><a href="https://travis-ci.com/thundergolfer/goodreads-sh"><img src="https://travis-ci.com/thundergolfer/goodreads-sh.svg?token=yHGWQ42iK2BPk1FjaUMc&branch=master"></a></p>
<p align="center">Command line interface for <a href="https://goodreads.com" > Goodreads</a>. Focuses on letting you quickly update your current reading activity, and giving you quick access to what your friends are reading.</p>
<p align="center">
<a href="https://i.imgur.com/der2fH7.gif"><img src="https://i.imgur.com/der2fH7.gif" width="600"/></a>
</p>



--- 

> ⚠️ **Notice:** This is only my 2nd Rust project, after [`slackify-markdown`](https://github.com/thundergolfer/slackify-markdown), and so while the project is functional the code is _not_ pretty.

## Installation

#### Homebrew

`coming soon` 

#### Manual Installtion

`coming soon`

#### [Required] Developer key
`goodreads-sh` requires your developer key and developer secret in order to read-write to the goodreads API. Obtaining them is fairly trivial.

1. Access your developer key and secret [here](https://www.goodreads.com/api/keys).
2. Copy your developer key and secret over to `goodreads-sh`'s config file. `~/.goodreads.toml`
```sh
developer_key = "<your_key_here>"
developer_secret = "<your_secret_here>"
```

Your config file should already be present in your home dir `~/.goodreads.toml` and if it's not, then run the command once without any options or create the file manually.

## Usage

```bash
USAGE:
    goodreads-sh <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    add-to-shelf    Add a book to an existing shelf [In Progress]
    auth            Setup OAuth for the CLI (1 time only)
    help            Prints this message or the help of the given subcommand(s)
    me              Show your User ID
    update          Update progress on a book you're currently reading
```

## Credit

- Kudos to [Danish Prakash](https://github.com/danishprakash/) for his implementation, [`goodreadsh`](https://github.com/danishprakash/goodreadsh), which I used and learned from while I developed this CLI. 🙏
