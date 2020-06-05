# Copyright (C) Brandon Waite 2020  - All Rights Reserved
# Unauthorized copying of this file, via any medium, is strictly prohibited
# Proprietary
# Updated by Brandon Waite, May 28 2020

function _scribe-recorder --on-event fish_preexec
    _scribe-release
end

function _scribe-release
    if [ -z "$argv" ]
        set argv "all"
    end
    if string match $argv "(recorder|all)"
        function _scribe-recorder
        end
    end
    if string match $argv "(search|all)"
        # TODO
    end
end
