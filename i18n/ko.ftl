License = 라이선스
version = 버전

usage = Usage: sushi [LONG OPTIONS] [OPTIONS] [SCRIPT] [ARGS]

shell_options =
    Shell options:
        -c COMMAND                 Execute COMMAND and exit
        -i                         Force interactive mode
        --version                  Display version information and exit
        --help                     Display this help message and exit
        --pipefall                 Return the status of the first failing command in a pipeline
        -B                         Enable brace expansion (equivalent to `set -B`)
        -e                         Exit immediately if any command returns a non-zero status

builtins =
    Builtin commands:
        cd                         Change the current directory
        pwd                        Print the current working directory
        exit                       Exit the shell
        source                     Read and execute commands from a file
        :                          No-op (does nothing)
        "."                        Source a file in the current shell
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
    Bourne shell variables:
        HOME                      User's home directory
        OPTARG                    Argument value for getopts
        OPTIND                    Index of next option to be processed by getopts
        PATH                      Search path for commands
        PS1                       Primary command prompt
        PS2                       Secondary command prompt
        _                         Last argument of previous command
        EPOCHREALTIME             Current time in seconds and nanoseconds (floating point)
        EPOCHSECONDS              Current time in seconds since epoch
        HISTFILE                  File where command history is saved
        HISTFILESIZE              Maximum number of lines in history file
        HOSTTYPE                  Machine hardware name
        LANG                      Locale setting
        LINENO                    Current line number in script
        MACHTYPE                  Machine type
        OLDPWD                    Previous working directory
        OSTYPE                    Operating system type
        PIPESTATUS                Exit status of last executed foreground pipeline
        PS4                       Prompt used by 'set -x' tracing
        PWD                       Current working directory
        RANDOM                    Random number generator seed
        SECONDS                   Number of seconds since shell started
        SHELL                     Path to the current shell executable
        SHLVL                     Shell nesting level
        SRANDOM                   Random seed (deprecated)

variables_bash =
    Bash-specific variables:
        BASHPID                   PID of this shell
        BASH_REMATCH              Array of regex matches after [[ =~ ]]
        BASH_SUBSHELL             Subshell level
        BASH_VERSINFO             Bash version info array
        BASH_VERSION              Bash version string

shopt =
    Shell options:
        dotglob                   Include hidden files (starting with .) in pathname expansions
        extglob                   Enable extended pattern matching operators
        progcomp                  Enable programmable command completion
        nullglob                  Allow patterns which match nothing to expand to null string

variables_born =
    Bourne shell variables:
        HOME                      User's home directory
        OPTARG                    Argument value for getopts
        OPTIND                    Index of next option to be processed by getopts
        PATH                      Search path for commands
        PS1                       Primary command prompt
        PS2                       Secondary command prompt
        _                         Last argument of previous command
        EPOCHREALTIME             Current time in seconds and nanoseconds (floating point)
        EPOCHSECONDS              Current time in seconds since epoch
        HISTFILE                  File where command history is saved
        HISTFILESIZE              Maximum number of lines in history file
        HOSTTYPE                  Machine hardware name
        LANG                      Locale setting
        LINENO                    Current line number in script
        MACHTYPE                  Machine type
        OLDPWD                    Previous working directory
        OSTYPE                    Operating system type
        PIPESTATUS                Exit status of last executed foreground pipeline
        PS4                       Prompt used by 'set -x' tracing
        PWD                       Current working directory
        RANDOM                    Random number generator seed
        SECONDS                   Number of seconds since shell started
        SHELL                     Path to the current shell executable
        SHLVL                     Shell nesting level
        SRANDOM                   Random seed (deprecated)

variables_bash =
    Bash-specific variables:
        BASHPID                   PID of this shell
        BASH_REMATCH              Array of regex matches after [[ =~ ]]
        BASH_SUBSHELL             Subshell level
        BASH_VERSINFO             Bash version info array
        BASH_VERSION              Bash version string
           
text_help = https://github.com/shellgei/rusty_bash

text_version =
    이것은 오픈 소스 소프트웨어입니다.
    원본 저작권 고지, 조건 목록 및 면책 조항이 유지되는 한,
    이 소프트웨어를 소스 또는 바이너리 형태로 수정하거나 하지 않고도
    자유롭게 사용, 수정 및 재배포할 수 있습니다.

    이 소프트웨어는 법이 허용하는 범위 내에서
    명시적이거나 묵시적인 어떠한 보증 없이 "있는 그대로" 제공됩니다.
