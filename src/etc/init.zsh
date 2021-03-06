# Copyright (C) Brandon Waite 2020  - All Rights Reserved
# Unauthorized copying of this file, via any medium, is strictly prohibited
# Proprietary
# Updated by Brandon Waite, May 28 2020

_scribe-recorder() {
    cmd=$( scribe record "$1" )
    if [[ "$cmd" == "release" || "$cmd" == "release-hooks" ]]; then
        _scribe-release
    fi
}
preexec_functions=(_scribe-recorder)
_scribe-history() {
    BUFFER=$(scribe search --interactive)
    CURSOR=${#BUFFER}
}
_SCRIBE_PREV_HISTORY_SEARCH=$(bindkey '^R' | cut -d' ' -f2)
zle -N _scribe-history
bindkey '^R' _scribe-history
_scribe-release() {
    args=$@
    if [ -z "$@" ]; then
        args="all"
    fi

    if [[ "$args" =~ "(recorder|all)" ]]; then
        echo 'Released recorder'
        preexec_functions=(${preexec_functions:#_scribe-recorder})
    fi

    if [[ "$args" =~ "(search|all)" ]]; then
        echo 'Released search'
        bindkey '^R' $_SCRIBE_PREV_HISTORY_SEARCH
    fi
}
