``` ebnf
(* Fileinfo communicates Chisel source file and line/column info *)
linecol = digit_dec , { digit_dec } , ":" , digit_dec , { digit_dec } ;
lineinfo = string, " ", linecol
info = "@" , "[" , lineinfo, { ",", lineinfo }, "]" ;

(* Type definitions *)
width = "<" , int_any , ">" ;
type_ground = "Clock" | "Reset" | "AsyncReset"
            |  type_ground_integral ;
type_ground_integral = ( "UInt" | "SInt" | "Analog" ) , [ width ] ;
type_aggregate = "{" , field , { field } , "}"
               | type , "[" , int_any , "]" ;
type_ref = ( "Probe" | "RWProbe" ) , "<", type , ">" ;
field = [ "flip" ] , id , ":" , type ;
type = type_hardware | type_ref ;
type_hardware = [ "const" ] , ( type_ground | type_aggregate ) ;

(* Primitive operations *)
primop_2expr_keyword =
    "add"  | "sub" | "mul" | "div" | "mod"
  | "lt"   | "leq" | "gt"  | "geq" | "eq" | "neq"
  | "dshl" | "dshr"
  | "and"  | "or"  | "xor" | "cat" ;
primop_2expr =
    primop_2expr_keyword , "(" , expr , "," , expr ")" ;
primop_1expr_keyword =
    "asUInt" | "asSInt" | "asClock" | "asAsyncReset" | "cvt"
  | "neg"    | "not"
  | "andr"   | "orr"    | "xorr" ;
primop_1expr =
    primop_1expr_keyword , "(" , expr , ")" ;
primop_1expr1int_keyword =
    "pad" | "shl" | "shr" | "head" | "tail" ;
primop_1expr1int =
    primop_1exrp1int_keyword , "(", expr , "," , int_any , ")" ;
primop_1expr2int_keyword =
    "bits" ;
primop_1expr2int =
    primop_1expr2int_keyword , "(" , expr , "," , int_any , "," , int_any , ")" ;
primop = primop_2expr | primop_1expr | primop_1expr1int | primop_1expr2int ;

(* Expression definitions *)
expr =
    ( "UInt" | "SInt" ) , [ width ] , "(" , int_any , ")"
  | reference
  | "mux" , "(" , expr , "," , expr , "," , expr , ")"
  | "read" , "(" , expr_ref , ")"
  | primop ;
static_reference = id
                 | static_reference , "." , id
                 | static_reference , "[" , int_any , "]" ;
reference = static_reference
          | reference , "[" , expr , "]" ;
expr_ref = ( "probe" | "rwprobe" ) , "(" , static_reference , ")"
           | static_reference ;

(* Memory *)
ruw =  "old" | "new" | "undefined" ;
memory = "mem" , id , ":" , [ info ] , newline , indent ,
           "data-type" , "=>" , type , newline ,
           "depth" , "=>" , int , newline ,
           "read-latency" , "=>" , int , newline ,
           "write-latency" , "=>" , int , newline ,
           "read-under-write" , "=>" , ruw , newline ,
           { "reader" , "=>" , id , newline } ,
           { "writer" , "=>" , id , newline } ,
           { "readwriter" , "=>" , id , newline } ,
         dedent ;

(* Force and Release *)
force_release =
    "force_initial" , "(" , expr_ref , "," , expr , ")"
  | "release_initial" , "(" , expr_ref , ")"
  | "force" , "(" , expr , "," , expr , "," , expr_ref , "," , expr , ")"
  | "release" , "(" , expr , "," , expr , "," , expr_ref , ")" ;

(* Statements *)
statement =
    "wire" , id , ":" , type , [ info ]
  | "reg" , id , ":" , type , expr ,
    [ "with" , ":" , "(" , "reset" , "=>" ,
      "(" , expr , "," , expr , ")", ")" ] ,
    [ info ]
  | "regreset" , id , ":" , type , "," , expr , "," , expr , "," , expr ,
    [info]
  | memory
  | "inst" , id , "of" , id , [ info ]
  | "node" , id , "=" , expr , [ info ]
  | reference , "<=" , expr , [ info ]
  | reference , "is invalid" , [ info ]
  | "attach(" , reference , { "," ,  reference } , ")" , [ info ]
  | "when" , expr , ":" [ info ] , newline ,
    indent , statement, { statement } , dedent ,
    [ "else" , ":" , indent , statement, { statement } , dedent ]
  | "stop(" , expr , "," , expr , "," , int , ")" , [ info ]
  | "printf(" , expr , "," , expr , "," , string_dq ,
    { expr } , ")" , [ ":" , id ] , [ info ]
  | "skip" , [ info ]
  | "define" , static_reference , "=" , expr_ref , [ info ]
  | force_release , [ info ]
  | "connect" , reference , "," , expr , [ info ]
  | "invalidate" , reference , [ info ] ;

(* Module definitions *)
port = ( "input" | "output" ) , id , ":" , type , [ info ] ;
module = "module" , id , ":" , [ info ] , newline , indent ,
           { port , newline } ,
           { statement , newline } ,
         dedent ;
type_param = int | string_dq | string_sq ;
extmodule = "extmodule" , id , ":" , [ info ] , newline , indent ,
              { port , newline } ,
              [ "defname" , "=" , id , newline ] ,
              { "parameter" , id , "=" , type_param , newline } ,
              { "ref" , static_reference , "is" ,
                '"' , static_reference , '"' , newline } ,
            dedent ;
intmodule = "intmodule" , id , ":" , [ info ] , newline , indent ,
              { port , newline } ,
              "intrinsic" , "=" , id , newline ,
              { "parameter" , "=" , ( int | string_dq ) , newline } ,
            dedent ;

(* In-line Annotations *)
annotations = "%" , "[" , json_array , "]" ;

(* Version definition *)
sem_ver = int , "."  , int , "." , int
version = "FIRRTL" , "version" , sem_ver ;

(* Circuit declarations *)
circuit_decl = module | extmodule | intmodule ;

(* Circuit definition *)
circuit =
  version , newline ,
  "circuit" , id , ":" , [ annotations ] , [ info ] , newline , indent ,
    { circuit_decl } ,
  dedent ;
```

