license = Leseni
version = toleo

usage = Usage: sushi [LONG OPTIONS] [OPTIONS] [SCRIPT] [ARGS]

options =
    Options:
        -c                         Execute COMMAND and exit
        -i                         Force interactive mode
        -l, --login                unsuported
        -r                         unsuported
        -s                         unsuported
        -D                         unsuported
        [-+]O                      unsuported
        --                         unsuported
        --debugger                 unsuported
        --dimp-po-strings          unsuported
        --help                     Display this help message and exit
        --init-file FILE           unsuported
        --rcfile FILE              unsuported
        --noediting                unsuported
        --noprofile                unsuported
        --norc                     unsuported
        --posix                    unsuported
        --restricted               unsuported
        -v, --verbose              unsuported
        --version                  Display version information and exit
        -e                         Exit immediately if a command returns non‑zero
        --pipefail                 Return status of first failing command in pipeline
        -B                         Enable brace expansion (equivalent to `set -B`)

builtins =
    Builtin commands:
        cd                         Change the current directory
        pwd                        Print the current working directory
        exit                       Exit the shell
        source                     Read and execute commands from a file
        :                          No-op (does nothing)
        "."                        Source a file in the current shell
        alias                      Define or display aliases
        break                      Exit from a loop
        builtin                    Execute a shell builtin, bypassing functions
        command                    Execute a command, ignoring shell functions
        continue                   Resume the next iteration of a loop
        eval                       Evaluate arguments as a shell command
        local                      Declare local variables inside functions
        return                     Return from a shell function
        false                      Do nothing, unsuccessfully
        true                       Do nothing, successfully
        shift                      Shift positional parameters
        unalias                    Remove aliases

shopt =
    Shell options:
        dotglob                   Include hidden files (starting with .) in pathname expansions
        extglob                   Enable extended pattern matching operators
        progcomp                  Enable programmable command completion
        nullglob                  Allow patterns which match nothing to expand to null string

variables_born =
    Born Shell Variables:
        CDPATH                    unsuported
        HOME                      User’s home directory
        IFS                       Internal Field Separator (partial support)
        MAIL                      unsuported
        MAILPATH                  unsuported
        OPTARG                    Argument value for the current option (getopts)
        OPTIND                    Index of the next argument to be processed by getopts
        PATH                      Search path for commands
        PS1                       Primary prompt string
        PS2                       Secondary prompt string

variables_bash =
    Bash Variables:
        _                         Last argument of the previous command
        BASH                      unsuported
        BASHOPTS                  unsuported
        BASHPID                   PID of the current Bash process
        BASH_ALIASES              unsuported
        BASH_ARGC                 unsuported
        BASH_ARGV                 unsuported
        BASH_ARGV0                unsuported
        BASH_CMDS                 unsuported
        BASH_COMMAND              unsuported
        BASH_COMPAT               unsuported
        BASH_ENV                  unsuported
        BASH_EXECUTION_STRING     unsuported
        BASH_LINENO               unsuported
        BASH_LOADABLES_PATH       unsuported
        BASH_REMATCH              Array of regex capture groups
        BASH_SOURCE               unsuported
        BASH_SUBSHELL             Current subshell level
        BASH_VERSINFO             Array with Bash version fields
        BASH_VERSION              Human‑readable Bash version
        BASH_XTRACEFD             unsuported
        CHILD_MAX                 unsuported
        COLUMNS                   unsuported
        COMP_CWORD                unsuported
        COMP_LINE                 unsuported
        COMP_POINT                unsuported
        COMP_TYPE                 unsuported
        COMP_KEY                  unsuported
        COMP_WORDBREAKS           unsuported
        COMP_WORDS                unsuported
        COMPREPLY                 unsuported
        COPROC                    unsuported
        DIRSTACK                  unsuported
        EMACS                     unsuported
        ENV                       unsuported
        EPOCHREALTIME             Epoch seconds with microseconds
        EPOCHSECONDS              Epoch seconds (integer)
        EUID                      unsuported
        EXECIGNORE                unsuported
        FCEDIT                    unsuported
        FIGNORE                   unsuported
        FUNCNAME                  unsuported
        FUNCNEST                  unsuported
        GLOBIGNORE                unsuported
        GROUPS                    unsuported
        histchars                 unsuported
        HISTCMD                   unsuported
        HISTCONTROL               unsuported
        HISTFILE                  Path to the history file
        HISTFILESIZE              Max lines kept in history file
        HISTIGNORE                unsuported
        HISTSIZE                  unsuported
        HISTTIMEFORMAT            unsuported
        HOSTFILE                  unsuported
        HOSTNAME                  unsuported
        HOSTTYPE                  Hardware platform string
        IGNOREEOF                 unsuported
        INPUTRC                   unsuported
        INSIDE_EMACS              unsuported
        LANG                      Current locale
        LC_ALL                    unsuported
        LC_COLLATE                unsuported
        LC_CTYPE                  unsuported
        LC_MESSAGES               unsuported
        LC_NUMERIC                unsuported
        LC_TIME                   unsuported
        LINENO                    Current script line number
        LINES                     unsuported
        MACHTYPE                  Machine type triple
        MAILCHECK                 unsuported
        MAPFILE                   unsuported
        OLDPWD                    Previous working directory
        OPTERR                    unsuported
        OSTYPE                    Operating‑system type
        PIPESTATUS                Exit statuses of the last pipeline
        POSIXLY_CORRECT           unsuported
        PPID                      unsuported
        PROMPT_COMMAND            unsuported
        PROMPT_DIRTRIM            unsuported
        PS0                       unsuported
        PS3                       unsuported
        PS4                       Debug prompt (used with set -x)
        PWD                       Current working directory
        RANDOM                    Pseudo‑random integer (0‑32767)
        READLINE_ARGUMENT         unsuported
        READLINE_LINE             unsuported
        READLINE_MARK             unsuported
        READLINE_POINT            unsuported
        REPLY                     unsuported
        SECONDS                   Seconds since the shell started
        SHELL                     Path to the user’s default shell
        SHELLOPTS                 unsuported
        SHLVL                     Shell nesting level
        SRANDOM                   64-bit cryptographic random
        TIMEFORMAT                unsuported
        TMOUT                     unsuported
        TMPDIR                    unsuported
        UID                       unsuported
    
    Beyond Bash feature: 
        branch display in prompt
           
text_help = Project homepage: https://github.com/shellgei/rusty_bash

text_version =
    Hii ni programu ya chanzo huria.
    Unaweza kuitumia, kuibadilisha, na kuisambaza tena kwa uhuru
    katika hali ya msimbo wa chanzo au iliyobainishwa, ukiwa umeifanyia
    mabadiliko au la, mradi taarifa ya hakimiliki, masharti na
    kanusho asilia zimedumishwa.

    PROGRAMU HII HUTOLEWA "KAMA ILIVYO", BILA DHAMANA YOYOTE,
    IWE IKO WAZI AU IMEFICHWA, KADRI INAVYORUHUSIWA KISHERIA.
