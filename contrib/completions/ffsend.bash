_ffsend() {
    local i cur prev opts cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${i}" in
            ffsend)
                cmd="ffsend"
                ;;
            
            complete)
                cmd+="__complete"
                ;;
            completion)
                cmd+="__completion"
                ;;
            completions)
                cmd+="__completions"
                ;;
            d)
                cmd+="__d"
                ;;
            dbg)
                cmd+="__dbg"
                ;;
            debug)
                cmd+="__debug"
                ;;
            del)
                cmd+="__del"
                ;;
            delete)
                cmd+="__delete"
                ;;
            down)
                cmd+="__down"
                ;;
            download)
                cmd+="__download"
                ;;
            e)
                cmd+="__e"
                ;;
            exist)
                cmd+="__exist"
                ;;
            exists)
                cmd+="__exists"
                ;;
            gen)
                cmd+="__gen"
                ;;
            generate)
                cmd+="__generate"
                ;;
            h)
                cmd+="__h"
                ;;
            help)
                cmd+="__help"
                ;;
            history)
                cmd+="__history"
                ;;
            i)
                cmd+="__i"
                ;;
            info)
                cmd+="__info"
                ;;
            information)
                cmd+="__information"
                ;;
            ls)
                cmd+="__ls"
                ;;
            p)
                cmd+="__p"
                ;;
            param)
                cmd+="__param"
                ;;
            parameter)
                cmd+="__parameter"
                ;;
            parameters)
                cmd+="__parameters"
                ;;
            params)
                cmd+="__params"
                ;;
            pass)
                cmd+="__pass"
                ;;
            password)
                cmd+="__password"
                ;;
            rm)
                cmd+="__rm"
                ;;
            u)
                cmd+="__u"
                ;;
            up)
                cmd+="__up"
                ;;
            upload)
                cmd+="__upload"
                ;;
            v)
                cmd+="__v"
                ;;
            ver)
                cmd+="__ver"
                ;;
            version)
                cmd+="__version"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        ffsend)
            opts=" -f -I -y -q -v -i -h -V -t -T -A -H  --force --no-interact --yes --quiet --verbose --incognito --help --version --timeout --transfer-timeout --api --basic-auth --history   debug delete download exists generate info parameters password upload version history help  dbg  del rm  d down  e exist  gen  i information  params param parameter  pass p  u up  ver v  h ls"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        
        ffsend__d)
            opts=" -e -h -V -f -I -y -q -v -i -p -o -t -T -A -H  --extract --help --version --force --no-interact --yes --quiet --verbose --incognito --password --output --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__dbg)
            opts=" -V -f -I -y -q -v -i -h -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --host --timeout --transfer-timeout --api --basic-auth --history  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --host)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -h)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__debug)
            opts=" -V -f -I -y -q -v -i -h -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --host --timeout --transfer-timeout --api --basic-auth --history  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --host)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -h)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__del)
            opts=" -h -V -f -I -y -q -v -i -o -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --owner --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__delete)
            opts=" -h -V -f -I -y -q -v -i -o -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --owner --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__down)
            opts=" -e -h -V -f -I -y -q -v -i -p -o -t -T -A -H  --extract --help --version --force --no-interact --yes --quiet --verbose --incognito --password --output --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__download)
            opts=" -e -h -V -f -I -y -q -v -i -p -o -t -T -A -H  --extract --help --version --force --no-interact --yes --quiet --verbose --incognito --password --output --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__e)
            opts=" -h -V -f -I -y -q -v -i -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__exist)
            opts=" -h -V -f -I -y -q -v -i -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__exists)
            opts=" -h -V -f -I -y -q -v -i -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__gen)
            opts=" -h -V -f -I -y -q -v -i -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --timeout --transfer-timeout --api --basic-auth --history   completions help  completion complete"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__generate)
            opts=" -h -V -f -I -y -q -v -i -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --timeout --transfer-timeout --api --basic-auth --history   completions help  completion complete"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__generate__complete)
            opts=" -h -V -f -I -y -q -v -i -o -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --output --timeout --transfer-timeout --api --basic-auth --history  <SHELL>... "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__generate__completion)
            opts=" -h -V -f -I -y -q -v -i -o -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --output --timeout --transfer-timeout --api --basic-auth --history  <SHELL>... "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__generate__completions)
            opts=" -h -V -f -I -y -q -v -i -o -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --output --timeout --transfer-timeout --api --basic-auth --history  <SHELL>... "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__generate__help)
            opts=" -h -V -f -I -y -q -v -i -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --timeout --transfer-timeout --api --basic-auth --history  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__h)
            opts=" -C -h -V -f -I -y -q -v -i -R -t -T -A -H  --clear --help --version --force --no-interact --yes --quiet --verbose --incognito --rm --timeout --transfer-timeout --api --basic-auth --history  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --rm)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -R)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__help)
            opts=" -h -V -f -I -y -q -v -i -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --timeout --transfer-timeout --api --basic-auth --history  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__history)
            opts=" -C -h -V -f -I -y -q -v -i -R -t -T -A -H  --clear --help --version --force --no-interact --yes --quiet --verbose --incognito --rm --timeout --transfer-timeout --api --basic-auth --history  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --rm)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -R)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__i)
            opts=" -h -V -f -I -y -q -v -i -o -p -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --owner --password --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__info)
            opts=" -h -V -f -I -y -q -v -i -o -p -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --owner --password --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__information)
            opts=" -h -V -f -I -y -q -v -i -o -p -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --owner --password --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__ls)
            opts=" -C -h -V -f -I -y -q -v -i -R -t -T -A -H  --clear --help --version --force --no-interact --yes --quiet --verbose --incognito --rm --timeout --transfer-timeout --api --basic-auth --history  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --rm)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -R)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__p)
            opts=" -P -h -V -f -I -y -q -v -i -p -o -t -T -A -H  --gen-passphrase --help --version --force --no-interact --yes --quiet --verbose --incognito --password --owner --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__param)
            opts=" -h -V -f -I -y -q -v -i -o -d -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --owner --download-limit --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --download-limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__parameter)
            opts=" -h -V -f -I -y -q -v -i -o -d -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --owner --download-limit --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --download-limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__parameters)
            opts=" -h -V -f -I -y -q -v -i -o -d -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --owner --download-limit --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --download-limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__params)
            opts=" -h -V -f -I -y -q -v -i -o -d -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --owner --download-limit --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --download-limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__pass)
            opts=" -P -h -V -f -I -y -q -v -i -p -o -t -T -A -H  --gen-passphrase --help --version --force --no-interact --yes --quiet --verbose --incognito --password --owner --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__password)
            opts=" -P -h -V -f -I -y -q -v -i -p -o -t -T -A -H  --gen-passphrase --help --version --force --no-interact --yes --quiet --verbose --incognito --password --owner --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__rm)
            opts=" -h -V -f -I -y -q -v -i -o -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --owner --timeout --transfer-timeout --api --basic-auth --history  <URL> "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --owner)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__u)
            opts=" -P -o -D -a -c -C -S -Q -V -f -I -y -q -v -i -p -d -e -h -n -t -T -A -H  --gen-passphrase --open --delete --archive --copy --copy-cmd --shorten --qrcode --help --version --force --no-interact --yes --quiet --verbose --incognito --password --download-limit --expiry-time --host --name --timeout --transfer-timeout --api --basic-auth --history  <FILE>... "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --download-limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --expiry-time)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -e)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --host)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -h)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__up)
            opts=" -P -o -D -a -c -C -S -Q -V -f -I -y -q -v -i -p -d -e -h -n -t -T -A -H  --gen-passphrase --open --delete --archive --copy --copy-cmd --shorten --qrcode --help --version --force --no-interact --yes --quiet --verbose --incognito --password --download-limit --expiry-time --host --name --timeout --transfer-timeout --api --basic-auth --history  <FILE>... "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --download-limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --expiry-time)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -e)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --host)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -h)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__upload)
            opts=" -P -o -D -a -c -C -S -Q -V -f -I -y -q -v -i -p -d -e -h -n -t -T -A -H  --gen-passphrase --open --delete --archive --copy --copy-cmd --shorten --qrcode --help --version --force --no-interact --yes --quiet --verbose --incognito --password --download-limit --expiry-time --host --name --timeout --transfer-timeout --api --basic-auth --history  <FILE>... "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --download-limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --expiry-time)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -e)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --host)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -h)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__v)
            opts=" -V -f -I -y -q -v -i -h -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --host --timeout --transfer-timeout --api --basic-auth --history  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --host)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -h)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__ver)
            opts=" -V -f -I -y -q -v -i -h -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --host --timeout --transfer-timeout --api --basic-auth --history  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --host)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -h)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
        ffsend__version)
            opts=" -V -f -I -y -q -v -i -h -t -T -A -H  --help --version --force --no-interact --yes --quiet --verbose --incognito --host --timeout --transfer-timeout --api --basic-auth --history  "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --host)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -h)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transfer-timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --basic-auth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --history)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -H)
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
    esac
}

complete -F _ffsend -o bashdefault -o default ffsend
