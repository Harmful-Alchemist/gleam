{module, test_small}.  %% version = 0

{exports, [{main,0},{module_info,0},{module_info,1},{t,3}]}.

{attributes, []}.

{labels, 14}.


{function, main, 0, 2}.
  {label,1}.
    {line,[{location,"build/dev/erlang/test_small/_gleam_artefacts/test_small.erl",
                     10}]}.
    {func_info,{atom,test_small},{atom,main},0}.
  {label,2}.
    {move,{literal,#{function => <<"main">>,gleam_error => todo,line => 2,
                     message => <<"This has not yet been implemented">>,
                     module => <<"test_small">>}},
          {x,0}}.
    {line,[{location,"build/dev/erlang/test_small/_gleam_artefacts/test_small.erl",
                     11}]}.
    {call_ext_only,1,{extfunc,erlang,error,1}}.


{function, t, 3, 4}.
  {label,3}.
    {line,[{location,"build/dev/erlang/test_small/_gleam_artefacts/test_small.erl",
                     18}]}.
    {func_info,{atom,test_small},{atom,t},3}.
  {label,4}.
    {test,is_eq_exact,{f,5},[{x,1},{atom,red}]}.
    {test,is_eq_exact,{f,5},[{x,2},{atom,black}]}.
    {move,{integer,1},{x,0}}.
    return.
  {label,5}.
    {test,is_eq_exact,{f,6},[{x,0},{atom,red}]}.
    {test,is_eq_exact,{f,6},[{x,1},{atom,black}]}.
    {move,{integer,2},{x,0}}.
    return.
  {label,6}.
    {select_val,{x,2},{f,9},{list,[{atom,black},{f,8},{atom,red},{f,7}]}}.
  {label,7}.
    {move,{integer,3},{x,0}}.
    return.
  {label,8}.
    {move,{integer,4},{x,0}}.
    return.
  {label,9}.
    {test_heap,4,3}.
    {put_tuple2,{x,0},{list,[{x,0},{x,1},{x,2}]}}.
    {line,[{location,"build/dev/erlang/test_small/_gleam_artefacts/test_small.erl",
                     19}]}.
    {case_end,{x,0}}.


{function, module_info, 0, 11}.
  {label,10}.
    {line,[]}.
    {func_info,{atom,test_small},{atom,module_info},0}.
  {label,11}.
    {move,{atom,test_small},{x,0}}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 13}.
  {label,12}.
    {line,[]}.
    {func_info,{atom,test_small},{atom,module_info},1}.
  {label,13}.
    {move,{x,0},{x,1}}.
    {move,{atom,test_small},{x,0}}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
