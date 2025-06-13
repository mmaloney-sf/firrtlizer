(* Circuit Definition *)
circuit =
  version , newline ,
  "circuit" , id , ":" , [ annotations ] , [ info ] , newline , indent ,
    { decl } ,
  dedent ;

(* Top-level Declarations *)
decl =
    decl_module
  | decl_extmodule
  | decl_layer
  | decl_formal
  | decl_type_alias ;

decl_module =
  [ "public" ], "module" , id , { enablelayer } , ":" , [ info ] ,
    newline , indent ,
    { port , newline } ,
    { statement , newline } ,
  dedent ;

