# Rusty Bash

may be a clone of Bash. 


## list of features

* :heavy_check_mark: :available
* :construction: :partially available (or having known bugs) 
* :no_good: : not implemented



### builtin commands

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| cd | :heavy_check_mark: | pwd | :heavy_check_mark: | read | :heavy_check_mark: |
| exit | :heavy_check_mark: | source | :heavy_check_mark: | set | :construction: | 
| shopt | :construction: |

### compound commands

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| if | :heavy_check_mark: | while | :heavy_check_mark: | () | :heavy_check_mark: | 
| {} | :heavy_check_mark: | case | :construction: | until | :no_good: | select | :no_good: | 
| for | :no_good: | (()) | :no_good: | [[]] | :no_good: | 


### control operator

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| \|\| | :heavy_check_mark: | && | :heavy_check_mark: | ; | :heavy_check_mark: |
| ;; | :heavy_check_mark: | \| | :heavy_check_mark: | & | :no_good: |
| \|& | :no_good: | 

### options 

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| i | :heavy_check_mark: | x | 🚧: | v | 🚧: |

### special parameters and position parameters


|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| * | :heavy_check_mark: | @ | :heavy_check_mark: | ? | :heavy_check_mark: |
| - | :heavy_check_mark: | 0 | :heavy_check_mark: | 1, 2, 3, ... | :heavy_check_mark: |
| # | :heavy_check_mark: | $ | :heavy_check_mark: | ! | :no_good: |
| _ | :no_good: |

### variables

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| OLDPWD| :heavy_check_mark: | PWD| :heavy_check_mark: | BASH | :no_good: |
| BASHOPTS| :no_good: | BASHPID| :no_good: | BASH_ALIASES| :no_good: |
| BASH_ARGC| :no_good: | BASH_ARGV| :no_good: | BASH_ARGV0| :no_good: |
| BASH_CMDS| :no_good: | BASH_COMMAND| :no_good: | BASH_EXECUTION_STRING| :no_good: |
| BASH_LINENO| :no_good: | BASH_LOADABLES_PATH| :no_good: | BASH_REMATCH| :no_good: |
| BASH_SOURCE| :no_good: | BASH_SUBSHELL| :no_good: | BASH_VERSINFO| :no_good: |
| BASH_VERSION| :no_good: | COMP_CWORD| :no_good: | COMP_KEY| :no_good: |
| COMP_LINE| :no_good: | COMP_POINT| :no_good: | COMP_TYPE| :no_good: |
| COMP_WORDBREAKS| :no_good: | COMP_WORDS| :no_good: | COPROC| :no_good: |
| DIRSTACK| :no_good: | EPOCHREALTIME| :no_good: | EPOCHSECONDS| :no_good: |
| EUID| :no_good: | FUNCNAME| :no_good: | GROUPS| :no_good: |
| HISTCMD| :no_good: | HOSTNAME| :no_good: | HOSTTYPE| :no_good: |
| LINENO| :no_good: | MACHTYPE| :no_good: | MAPFILE| :no_good: |
| OPTARG| :no_good: | OPTIND| :no_good: | OSTYPE| :no_good: |
| PIPESTATUS| :no_good: | PPID| :no_good: | RANDOM| :no_good: |
| READLINE_LINE| :no_good: | READLINE_POINT| :no_good: | REPLY| :no_good: |
| SECONDS| :no_good: | SHELLOPTS| :no_good: | SHLVL| :no_good: |
| UID| :no_good: | BASH_COMPAT| :no_good: | BASH_ENV| :no_good: |
| BASH_XTRACEFD| :no_good: | CDPATH| :no_good: | CHILD_MAX| :no_good: |
| COLUMNS| :no_good: | COMPREPLY| :no_good: | EMACS | :no_good: |
| ENV| :no_good: | EXECIGNORE| :no_good: | FCEDIT| :no_good: |
| FIGNORE| :no_good: | FUNCNEST| :no_good: | GLOBIGNORE| :no_good: |
| HISTCONTROL| :no_good: | HISTFILE| :no_good: | HISTFILESIZE| :no_good: |
| HISTIGNORE| :no_good: | HISTSIZE| :no_good: | HISTTIMEFORMAT| :no_good: |
| HOME| :no_good: | HOSTFILE| :no_good: | IFS| :construction: |
| IGNOREEOF| :no_good: | INPUTRC| :no_good: | INSIDE_EMACS| :no_good: |
| LANG| :no_good: | LC_ALL| :no_good: | LC_COLLATE| :no_good: |
| LC_CTYPE| :no_good: | LC_MESSAGES| :no_good: | LC_NUMERIC| :no_good: |
| LC_TIME| :no_good: | LINES| :no_good: | MAIL| :no_good: |
| MAILCHECK| :no_good: | MAILPATH| :no_good: | OPTERR| :no_good: |
| PATH| :heavy_check_mark: | POSIXLY_CORRECT| :no_good: | PROMPT_COMMAND| :no_good: |
| PROMPT_DIRTRIM| :no_good: | PS0| :no_good: | PS1| :no_good: |
| PS2| :no_good: | PS3| :no_good: | PS4| :no_good: |
| SHELL| :no_good: | TIMEFORMAT| :no_good: | TMOUT| :no_good: |
| TMPDIR| :no_good: | auto_resume| :no_good: | histchars| :no_good: |

### others 

|features | status |
|-------------------|----|
| coproc | :no_good: |
