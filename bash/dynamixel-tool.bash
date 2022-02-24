_dynamixel-tool-models() {
    local P
    if echo "${COMP_WORDS[@]}" | egrep '\-P\s*2|--protocol[= ]*2'>/dev/null; then
        P='-P2'
    fi
    echo $("${COMP_WORDS[0]}" ${P} list-models 2>/dev/null)
}

_dynamixel-tool-regs() {
    local P M regs models

    if echo "${COMP_WORDS[@]}" | egrep '\-P\s*2|--protocol[= ]*2'>/dev/null; then
        P='-P2'
    fi

    M=$(echo $1 | cut -d / -f 1)

    if [[ $("${COMP_WORDS[0]}" ${P} list-models 2>/dev/null | egrep ^$M | wc -l) == 1 ]]; then
        M=$("${COMP_WORDS[0]}" ${P} list-models 2>/dev/null | egrep ^$M)
    fi

    regs="$("${COMP_WORDS[0]}" ${P} list-registers $M 2>/dev/null | cut -c 11- | sed s,^,$M/,)"

    if [[ -n $regs ]]; then
        echo "$regs"
    else
        echo $("${COMP_WORDS[0]}" ${P} list-models 2>/dev/null)
    fi
}

_dynamixel-tool() {
    local i cur prev opts cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${i}" in
            "$1")
                cmd="dynamixel__tool"
                ;;
            help)
                cmd+="__help"
                ;;
            list-models)
                cmd+="__list__models"
                ;;
            list-registers)
                cmd+="__list__registers"
                ;;
            read-bytes)
                cmd+="__read__bytes"
                ;;
            read-reg)
                cmd+="__read__reg"
                ;;
            read-uint16)
                cmd+="__read__uint16"
                ;;
            read-uint32)
                cmd+="__read__uint32"
                ;;
            read-uint8)
                cmd+="__read__uint8"
                ;;
            scan)
                cmd+="__scan"
                ;;
            write-bytes)
                cmd+="__write__bytes"
                ;;
            write-reg)
                cmd+="__write__reg"
                ;;
            write-uint16)
                cmd+="__write__uint16"
                ;;
            write-uint32)
                cmd+="__write__uint32"
                ;;
            write-uint8)
                cmd+="__write__uint8"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        dynamixel__tool)
            opts="-h -V -f -d -p -b -r -j -P --help --version --force --debug --port --baudrate --retries --json --protocol list-models list-registers scan read-uint8 read-uint16 read-uint32 read-bytes read-reg write-uint8 write-uint16 write-uint32 write-bytes write-reg help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --port)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --baudrate)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -b)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --retries)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -r)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --protocol)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -P)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__help)
            opts="<SUBCOMMAND>..."
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__list__models)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__list__registers)
            opts="-h --help $(_dynamixel-tool-models)"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__read__bytes)
            opts="-h --help <IDS> <ADDRESS> <COUNT>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__read__reg)
            opts="-h --help <IDS> <REG>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi

            if [[ ${COMP_WORDS[$((COMP_CWORD-2))]} == read-reg ]]; then
                opts=$(_dynamixel-tool-regs "${cur}")
            fi

            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__read__uint16)
            opts="-h --help <IDS> <ADDRESS>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__read__uint32)
            opts="-h --help <IDS> <ADDRESS>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__read__uint8)
            opts="-h --help <IDS> <ADDRESS>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__scan)
            opts="-h --help <SCAN_START> <SCAN_END>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__write__bytes)
            opts="-h --help <IDS> <ADDRESS> <VALUES>..."
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__write__reg)
            opts="-h --help <IDS> <REG> <VALUE>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi

            if [[ ${COMP_WORDS[$((COMP_CWORD-2))]} == write-reg ]]; then
                opts=$(_dynamixel-tool-regs "${cur}")
            fi

            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__write__uint16)
            opts="-h --help <IDS> <ADDRESS> <VALUE>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__write__uint32)
            opts="-h --help <IDS> <ADDRESS> <VALUE>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        dynamixel__tool__write__uint8)
            opts="-h --help <IDS> <ADDRESS> <VALUE>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

complete -F _dynamixel-tool -o bashdefault -o default dynamixel-tool
