license = ライセンス
version = バージョン

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
    これはオープンソースソフトウェアです。
    このソフトウェアは、オリジナルの著作権表示、条件の一覧、
    免責事項が保持されている限り、ソースまたはバイナリ形式で、
    修正の有無にかかわらず、自由に使用、変更、再配布できます。

    本ソフトウェアは、法律で許される範囲において、
    明示的または黙示的ないかなる保証もなく「現状のまま」提供されます。
