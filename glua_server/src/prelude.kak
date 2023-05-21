# Glua will raise any error in kakoune format to stdout if any accures
provide-module glua %{
    decl str glua_exe "%val{config}/testing/target/debug/glua"
    decl str glua_root "%val{config}/testing/"

	# Request with lua chunck and arguments must contain:
    # 1.socket path , 2.session, 3.client, 4.chunck
    # Order matters, otherwise error will be raised
    def glua-eval -override -params 1.. %{
        eval %sh{ "$kak_opt_glua_exe" eval "$kak_opt_glua_root" "$kak_session" "$kak_client" "$@"}
    }

	# `eval` async by default, so will not block editor while chunck executing.
    # For sync evalutation use `synceval` which will wait for socket to respond after chunck execution.
    def glua-eval -override -params 1.. %{
        eval %sh{ "$kak_opt_glua_exe" eval "$kak_opt_glua_root" "$kak_session" "$kak_client" "$@"}
    }

	# Spawn and kill commands without provided path will look into /tmp and
    # will spawn/kill directory with socket there.
    def glua-eval-async -override -params 1.. %{
        eval %sh{ "$kak_opt_glua_exe" asynceval "$kak_opt_glua_root" "$kak_session" "$kak_client" "$@"}
    }

	# `Spawn` command creates directory in provided path which will contain
    # unix socket, to which requests with chunck and arguments will be sent and
    # file which contains Glua daemon pid.
    # This directory will also be used for storing temporary files created by
    # Glua functions, 'temp_fifo()' for example. 
    def glua-start -override %{
    eval %sh{ "$kak_opt_glua_exe" spawn "$kak_opt_glua_root" }
    }

    # After creating root directory, `Spawn` command will
    # store root path into %opt{GLUA_last_spawned}
    def glua-spawn-it -override -params ..1 %{
        eval %sh{ "$kak_opt_glua_exe" spawn "$1"}
    }

    # If provided path for `kill` command is directory, Glua will try to find and
    # kill EVRY socket in this directory.
    def glua-kill-it -override -params ..1 %{
        eval %sh{ "$kak_opt_glua_exe" kill "$1"}
    }

    # Lua State will be updated till socket lives, means
    # any value/function declared as global will be saved,
    # and so can be used from any client or session after chunck
    # in which tat variable was declared.

	# You can also send chunck to a different socket, remember
    # that each of them have their own globals.

    # Special Glua functions stored in `kak` global table:
    # 		kak.eval(<cmd:Str>) - does the same as kakoune's cmd line
    # 		kak.send_to(<session:Str>, <cmd:Str>) - does the same as `kak -p <session> <cmd>`
}
