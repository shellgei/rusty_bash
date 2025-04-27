# setup of bash-completion

- 20250427 Ryuichi Ueda
    - bluesky: @ueda.tech (https://bsky.app/profile/ueda.tech)

## contents

- Linux
    - with command_not_found
- macOS

## Linux (Ubuntu 24.04 or some versions near 24.04)

  The following three lines are minimally required for bash-completion
at the bottom of `~/.sushrc` if you are using v1.1.4 or a version near v1.1.4. 

```bash
source /usr/share/bash-completion/bash_completion
_comp_complete_load scp #for completion of rsync
complete -d cd
```

With this three lines, you can use completion for various commands
including `git`. 

  The first line calls the main script of Bash-completion, which usually
exists in `/usr/share/bash-completion/`. If you can't find the path,
please search it by `find` and change the path.

  The second line is required since a warning disturbs completion for `rsync`.
This problem is solved if the completion function for `scp` is load. 
Since this happens owing to a bug of Rusty Bash, it should be removed someday. 
Someday...

  The third line is also necessary to avoid a problem. This line resets
the completion method for `cd` to directory completion since it was set 
to function completion after `bash_completion` for some reason.

### with `command_not_found`

  You can also use `command_not_found`.  Before the three lines for
bash-completion, please add the following function. The path should be
changed if `command-not-found` script exists in the different directory. 

```bash
command_not_found_handle() {
        if [ -e /usr/lib/command-not-found ] ; then
                /usr/lib/command-not-found -- "$1"
        fi
}
```

For some reason, this definition has to be written BEFORE the call of
bash-completion. 

## macOS (Sequoia 15.3.1 or some versions near it)

  In macOS, the three lines for bash-completion are required at the bottom
of `~/.sushrc`. Moreover, we may have to install bash-completion by ourselves. 

  When you are using homebrew, you can do it with the following command.

```bash
🍣 brew install bash-completion
```

Then you can find `bash_completion` script in a directory under `/opt/homebrew/Cellar`
like this. 

```bash
🍣 find /opt/homebrew/Cellar/ | grep bash_completion$
/opt/homebrew/Cellar//bash-completion/1.3_3/etc/bash_completion
```

Please use the path found as above instead of the path used in the Linux example.

