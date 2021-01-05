
// ** prelude from <inline-prelude>

// ================================================================================
// Notation

// This files contains a Handlebars Rust template for the prover's Boogie prelude.
// The template language constructs allow the prelude to adjust the actual content
// to multiple options. We only use a few selected template constructs which are
// mostly self-explaining. See the handlebars crate documentation for more information.
//
// The object passed in as context for templates is the struct cli::Options and its
// sub-structs.

// ================================================================================
// Domains

// Debug tracking
// --------------

// Debug tracking is used to inject information used for model analysis. The generated code emits statements
// like this:
//
//     assume $DebugTrackLocal(file_id, byte_index, var_idx, $Value);
//
// While those tracking assumptions are trivially true for the provers logic, the solver (at least Z3)
// will construct a function mapping which appears in the model, e.g.:
//
//     $DebugTrackLocal -> {
//         1 440 0 (Vector (ValueArray |T@[Int]$Value!val!0| 0)) -> true
//         1 533 1 ($Integer 1) -> true
//         ...
//         else -> true
//     }
//
// This information can then be read out from the model.


// Tracks debug information for a parameter, local or a return parameter. Return parameter indices start at
// the overall number of locals (including parameters).
function $DebugTrackLocal(file_id: int, byte_index:  int, var_idx: int, $Value: $Value) : bool {
  true
}

// Tracks at which location a function was aborted.
function $DebugTrackAbort(file_id: int, byte_index: int, code: int) : bool {
  true
}

// Tracks the $Value of a specification (sub-)expression.
function $DebugTrackExp(module_id: int, node_id: int, $Value: $Value) : $Value { $Value }


// Path type
// ---------

type {:datatype} $Path;
function {:constructor} $Path(p: [int]int, size: int): $Path;
const $EmptyPath: $Path;
axiom size#$Path($EmptyPath) == 0;

function {:inline} $path_index_at(p: $Path, i: int): int {
    p#$Path(p)[i]
}

// Type Values
// -----------

type $TypeName;
type $FieldName = int;
type $LocalName;
type {:datatype} $TypeValue;
function {:constructor} $BooleanType() : $TypeValue;
function {:constructor} $IntegerType() : $TypeValue;
function {:constructor} $AddressType() : $TypeValue;
function {:constructor} $StrType() : $TypeValue;
function {:constructor} $VectorType(t: $TypeValue) : $TypeValue;
function {:constructor} $StructType(name: $TypeName, ts: $TypeValueArray) : $TypeValue;
function {:constructor} $TypeType(): $TypeValue;
function {:constructor} $ErrorType() : $TypeValue;

function {:inline} $DefaultTypeValue() : $TypeValue { $ErrorType() }
function {:builtin "MapConst"} $MapConstTypeValue(tv: $TypeValue): [int]$TypeValue;

type {:datatype} $TypeValueArray;
function {:constructor} $TypeValueArray(v: [int]$TypeValue, l: int): $TypeValueArray;
const $EmptyTypeValueArray: $TypeValueArray;
axiom l#$TypeValueArray($EmptyTypeValueArray) == 0;
axiom v#$TypeValueArray($EmptyTypeValueArray) == $MapConstTypeValue($DefaultTypeValue());



// Values
// ------

type {:datatype} $Value;

const $MAX_U8: int;
axiom $MAX_U8 == 255;
const $MAX_U64: int;
axiom $MAX_U64 == 18446744073709551615;
const $MAX_U128: int;
axiom $MAX_U128 == 340282366920938463463374607431768211455;

function {:constructor} $Boolean(b: bool): $Value;
function {:constructor} $Integer(i: int): $Value;
function {:constructor} $Address(a: int): $Value;
function {:constructor} $Vector(v: $ValueArray): $Value; // used to both represent Move Struct and Vector
function {:constructor} $Range(lb: $Value, ub: $Value): $Value;
function {:constructor} $Type(t: $TypeValue): $Value;
function {:constructor} $Error(): $Value;

function {:inline} $DefaultValue(): $Value { $Error() }
function {:builtin "MapConst"} $MapConstValue(v: $Value): [int]$Value;

function {:inline} $IsValidU8(v: $Value): bool {
  is#$Integer(v) && i#$Integer(v) >= 0 && i#$Integer(v) <= $MAX_U8
}

function {:inline} $IsValidU8Vector(vec: $Value): bool {
  $Vector_$is_well_formed(vec)
  && (forall i: int :: {$select_vector(vec, i)} 0 <= i && i < $vlen(vec) ==> $IsValidU8($select_vector(vec, i)))
}

function {:inline} $IsValidU64(v: $Value): bool {
  is#$Integer(v) && i#$Integer(v) >= 0 && i#$Integer(v) <= $MAX_U64
}

function {:inline} $IsValidU128(v: $Value): bool {
  is#$Integer(v) && i#$Integer(v) >= 0 && i#$Integer(v) <= $MAX_U128
}

function {:inline} $IsValidNum(v: $Value): bool {
  is#$Integer(v)
}


// Value Array
// -----------





// This is the implementation of $ValueArray using integer maps

type {:datatype} $ValueArray;

function {:constructor} $ValueArray(v: [int]$Value, l: int): $ValueArray;

function $EmptyValueArray(): $ValueArray;
axiom l#$ValueArray($EmptyValueArray()) == 0;
axiom v#$ValueArray($EmptyValueArray()) == $MapConstValue($Error());

function {:inline} $ReadValueArray(a: $ValueArray, i: int): $Value {
    (
        v#$ValueArray(a)[i]
    )
}

function {:inline} $LenValueArray(a: $ValueArray): int {
    (
        l#$ValueArray(a)
    )
}

function {:inline} $RemoveValueArray(a: $ValueArray): $ValueArray {
    (
        var l := l#$ValueArray(a) - 1;
        $ValueArray(
            (lambda i: int ::
                if i >= 0 && i < l then v#$ValueArray(a)[i] else $DefaultValue()),
            l
        )
    )
}

function {:inline} $SingleValueArray(v: $Value): $ValueArray {
    $ValueArray($MapConstValue($DefaultValue())[0 := v], 1)
}

function {:inline} $RemoveIndexValueArray(a: $ValueArray, i: int): $ValueArray {
    (
        var l := l#$ValueArray(a) - 1;
        $ValueArray(
            (lambda j: int ::
                if j >= 0 && j < l then
                    if j < i then v#$ValueArray(a)[j] else v#$ValueArray(a)[j+1]
                else $DefaultValue()),
            l
        )
    )
}

function {:inline} $ConcatValueArray(a1: $ValueArray, a2: $ValueArray): $ValueArray {
    (
        var l1, m1, l2, m2 := l#$ValueArray(a1), v#$ValueArray(a1), l#$ValueArray(a2), v#$ValueArray(a2);
        $ValueArray(
            (lambda i: int ::
                if i >= 0 && i < l1 + l2 then
                    if i < l1 then m1[i] else m2[i - l1]
                else
                    $DefaultValue()),
            l1 + l2)
    )
}

function {:inline} $ReverseValueArray(a: $ValueArray): $ValueArray {
    (
        var l := l#$ValueArray(a);
        $ValueArray(
            (lambda i: int :: if 0 <= i && i < l then v#$ValueArray(a)[l - i - 1] else $DefaultValue()),
            l
        )
    )
}

function {:inline} $SliceValueArray(a: $ValueArray, i: int, j: int): $ValueArray { // return the sliced vector of a for the range [i, j)
    $ValueArray((lambda k:int :: if 0 <= k && k < j-i then v#$ValueArray(a)[i+k] else $DefaultValue()), (if j-i < 0 then 0 else j-i))
}

function {:inline} $ExtendValueArray(a: $ValueArray, elem: $Value): $ValueArray {
    (var len := l#$ValueArray(a);
     $ValueArray(v#$ValueArray(a)[len := elem], len + 1))
}

function {:inline} $UpdateValueArray(a: $ValueArray, i: int, elem: $Value): $ValueArray {
    $ValueArray(v#$ValueArray(a)[i := elem], l#$ValueArray(a))
}

function {:inline} $SwapValueArray(a: $ValueArray, i: int, j: int): $ValueArray {
    $ValueArray(v#$ValueArray(a)[i := v#$ValueArray(a)[j]][j := v#$ValueArray(a)[i]], l#$ValueArray(a))
}

function {:inline} $IsEmpty(a: $ValueArray): bool {
    l#$ValueArray(a) == 0
}

// All invalid elements of array are DefaultValue. This is useful in specialized
// cases. This is used to defined normalization for $Vector
function {:inline} $IsNormalizedValueArray(a: $ValueArray, len: int): bool {
    (forall i: int :: i < 0 || i >= len ==> v#$ValueArray(a)[i] == $DefaultValue())
}


 //end of backend.vector_using_sequences


// Stratified Functions on Values
// ------------------------------

// TODO: templatize this or move it back to the translator. For now we
//   prefer to handcode this so its easier to evolve the model independent of the
//   translator.

const $StratificationDepth: int;
axiom $StratificationDepth == 4;



// Generate a stratified version of IsEqual for depth of 4.

function  $IsEqual_stratified(v1: $Value, v2: $Value): bool {
    (v1 == v2) ||
    (is#$Vector(v1) &&
     is#$Vector(v2) &&
     $vlen(v1) == $vlen(v2) &&
     (forall i: int :: 0 <= i && i < $vlen(v1) ==> $IsEqual_level1($select_vector(v1,i), $select_vector(v2,i))))
}

function  $IsEqual_level1(v1: $Value, v2: $Value): bool {
    (v1 == v2) ||
    (is#$Vector(v1) &&
     is#$Vector(v2) &&
     $vlen(v1) == $vlen(v2) &&
     (forall i: int :: 0 <= i && i < $vlen(v1) ==> $IsEqual_level2($select_vector(v1,i), $select_vector(v2,i))))
}

function  $IsEqual_level2(v1: $Value, v2: $Value): bool {
    (v1 == v2) ||
    (is#$Vector(v1) &&
     is#$Vector(v2) &&
     $vlen(v1) == $vlen(v2) &&
     (forall i: int :: 0 <= i && i < $vlen(v1) ==> $IsEqual_level3($select_vector(v1,i), $select_vector(v2,i))))
}

function {:inline} $IsEqual_level3(v1: $Value, v2: $Value): bool {
    v1 == v2
}


function {:inline} $IsEqual(v1: $Value, v2: $Value): bool {
    $IsEqual_stratified(v1, v2)
}



// Generate stratified ReadValue for the depth of 4.


function  $ReadValue_stratified(p: $Path, v: $Value) : $Value {
    if (0 == size#$Path(p)) then
        v
    else
        $ReadValue_level1(p, $select_vector(v,$path_index_at(p, 0)))
}

function  $ReadValue_level1(p: $Path, v: $Value) : $Value {
    if (1 == size#$Path(p)) then
        v
    else
        $ReadValue_level2(p, $select_vector(v,$path_index_at(p, 1)))
}

function  $ReadValue_level2(p: $Path, v: $Value) : $Value {
    if (2 == size#$Path(p)) then
        v
    else
        $ReadValue_level3(p, $select_vector(v,$path_index_at(p, 2)))
}

function {:inline} $ReadValue_level3(p: $Path, v: $Value): $Value {
    v
}


function {:inline} $ReadValue(p: $Path, v: $Value): $Value {
    $ReadValue_stratified(p, v)
}

// Generate stratified $UpdateValue for the depth of 4.


function  $UpdateValue_stratified(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    (var poffset := offset + 0;
    if (poffset == size#$Path(p)) then
        new_v
    else
        $update_vector(v, $path_index_at(p, poffset),
                       $UpdateValue_level1(p, offset, $select_vector(v,$path_index_at(p, poffset)), new_v)))
}

function  $UpdateValue_level1(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    (var poffset := offset + 1;
    if (poffset == size#$Path(p)) then
        new_v
    else
        $update_vector(v, $path_index_at(p, poffset),
                       $UpdateValue_level2(p, offset, $select_vector(v,$path_index_at(p, poffset)), new_v)))
}

function  $UpdateValue_level2(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    (var poffset := offset + 2;
    if (poffset == size#$Path(p)) then
        new_v
    else
        $update_vector(v, $path_index_at(p, poffset),
                       $UpdateValue_level3(p, offset, $select_vector(v,$path_index_at(p, poffset)), new_v)))
}

function {:inline} $UpdateValue_level3(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    new_v
}


function {:inline} $UpdateValue(p: $Path, offset: int, v: $Value, new_v: $Value): $Value {
    $UpdateValue_stratified(p, offset, v, new_v)
}

// Generate stratified $IsPathPrefix for the depth of 4.


function  $IsPathPrefix_stratified(p1: $Path, p2: $Path): bool {
    if (0 == size#$Path(p1)) then
        true
    else if (p#$Path(p1)[0] == p#$Path(p2)[0]) then
        $IsPathPrefix_level1(p1, p2)
    else
        false
}

function  $IsPathPrefix_level1(p1: $Path, p2: $Path): bool {
    if (1 == size#$Path(p1)) then
        true
    else if (p#$Path(p1)[1] == p#$Path(p2)[1]) then
        $IsPathPrefix_level2(p1, p2)
    else
        false
}

function  $IsPathPrefix_level2(p1: $Path, p2: $Path): bool {
    if (2 == size#$Path(p1)) then
        true
    else if (p#$Path(p1)[2] == p#$Path(p2)[2]) then
        $IsPathPrefix_level3(p1, p2)
    else
        false
}

function {:inline} $IsPathPrefix_level3(p1: $Path, p2: $Path): bool {
    true
}


function {:inline} $IsPathPrefix(p1: $Path, p2: $Path): bool {
    $IsPathPrefix_stratified(p1, p2)
}

// Generate stratified $ConcatPath for the depth of 4.


function  $ConcatPath_stratified(p1: $Path, p2: $Path): $Path {
    if (0 == size#$Path(p2)) then
        p1
    else
        $ConcatPath_level1($Path(p#$Path(p1)[size#$Path(p1) := p#$Path(p2)[0]], size#$Path(p1) + 1), p2)
}

function  $ConcatPath_level1(p1: $Path, p2: $Path): $Path {
    if (1 == size#$Path(p2)) then
        p1
    else
        $ConcatPath_level2($Path(p#$Path(p1)[size#$Path(p1) := p#$Path(p2)[1]], size#$Path(p1) + 1), p2)
}

function  $ConcatPath_level2(p1: $Path, p2: $Path): $Path {
    if (2 == size#$Path(p2)) then
        p1
    else
        $ConcatPath_level3($Path(p#$Path(p1)[size#$Path(p1) := p#$Path(p2)[2]], size#$Path(p1) + 1), p2)
}

function {:inline} $ConcatPath_level3(p1: $Path, p2: $Path): $Path {
    p1
}


function {:inline} $ConcatPath(p1: $Path, p2: $Path): $Path {
    $ConcatPath_stratified(p1, p2)
}

// Vector related functions on Values
// ----------------------------------

function {:inline} $vlen(v: $Value): int {
    $LenValueArray(v#$Vector(v))
}

// Check that all invalid elements of vector are DefaultValue
function {:inline} $is_normalized_vector(v: $Value): bool {
    $IsNormalizedValueArray(v#$Vector(v), $vlen(v))
}

// Sometimes, we need the length as a Value, not an int.
function {:inline} $vlen_value(v: $Value): $Value {
    $Integer($vlen(v))
}
function {:inline} $mk_vector(): $Value {
    $Vector($EmptyValueArray())
}
function {:inline} $push_back_vector(v: $Value, elem: $Value): $Value {
    $Vector($ExtendValueArray(v#$Vector(v), elem))
}
function {:inline} $pop_back_vector(v: $Value): $Value {
    $Vector($RemoveValueArray(v#$Vector(v)))
}
function {:inline} $single_vector(v: $Value): $Value {
    $Vector($SingleValueArray(v))
}
function {:inline} $append_vector(v1: $Value, v2: $Value): $Value {
    $Vector($ConcatValueArray(v#$Vector(v1), v#$Vector(v2)))
}
function {:inline} $reverse_vector(v: $Value): $Value {
    $Vector($ReverseValueArray(v#$Vector(v)))
}
function {:inline} $update_vector(v: $Value, i: int, elem: $Value): $Value {
    $Vector($UpdateValueArray(v#$Vector(v), i, elem))
}
// $update_vector_by_value requires index to be a Value, not int.
function {:inline} $update_vector_by_value(v: $Value, i: $Value, elem: $Value): $Value {
    $Vector($UpdateValueArray(v#$Vector(v), i#$Integer(i), elem))
}
function {:inline} $select_vector(v: $Value, i: int) : $Value {
    $ReadValueArray(v#$Vector(v), i)
}
// $select_vector_by_value requires index to be a Value, not int.
function {:inline} $select_vector_by_value(v: $Value, i: $Value) : $Value {
    $select_vector(v, i#$Integer(i))
}
function {:inline} $swap_vector(v: $Value, i: int, j: int): $Value {
    $Vector($SwapValueArray(v#$Vector(v), i, j))
}
function {:inline} $slice_vector(v: $Value, r: $Value) : $Value {
    $Vector($SliceValueArray(v#$Vector(v), i#$Integer(lb#$Range(r)), i#$Integer(ub#$Range(r))))
}
function {:inline} $InVectorRange(v: $Value, i: int): bool {
    i >= 0 && i < $vlen(v)
}
function {:inline} $remove_vector(v: $Value, i:int): $Value {
    $Vector($RemoveIndexValueArray(v#$Vector(v), i))
}
function {:inline} $contains_vector(v: $Value, e: $Value): bool {
    (exists i:int :: 0 <= i && i < $vlen(v) && $IsEqual($select_vector(v,i), e))
}

function {:inline} $InRange(r: $Value, i: int): bool {
   i#$Integer(lb#$Range(r)) <= i && i < i#$Integer(ub#$Range(r))
}


// ============================================================================================
// Memory

type {:datatype} $Location;

// A global resource location within the statically known resource type's memory.
// `ts` are the type parameters for the outer type, and `a` is the address.
function {:constructor} $Global(ts: $TypeValueArray, a: int): $Location;

// A local location. `i` is the unique index of the local.
function {:constructor} $Local(i: int): $Location;

// The location of a reference outside of the verification scope, for example, a `&mut` parameter
// of the function being verified. References with these locations don't need to be written back
// when mutation ends.
function {:constructor} $Param(i: int): $Location;


// A mutable reference which also carries its current value. Since mutable references
// are single threaded in Move, we can keep them together and treat them as a value
// during mutation until the point they are stored back to their original location.
type {:datatype} $Mutation;
function {:constructor} $Mutation(l: $Location, p: $Path, v: $Value): $Mutation;
const $DefaultMutation: $Mutation;

// Representation of memory for a given type. The maps take the content of a Global location.
type {:datatype} $Memory;
function {:constructor} $Memory(domain: [$TypeValueArray, int]bool, contents: [$TypeValueArray, int]$Value): $Memory;

function {:inline} $Memory__is_well_formed(m: $Memory): bool {
    true
}

function {:builtin "MapConst"} $ConstMemoryDomain(v: bool): [$TypeValueArray, int]bool;
function {:builtin "MapConst"} $ConstMemoryContent(v: $Value): [$TypeValueArray, int]$Value;
axiom $ConstMemoryDomain(false) == (lambda ta: $TypeValueArray, i: int :: false);
axiom $ConstMemoryDomain(true) == (lambda ta: $TypeValueArray, i: int :: true);

const $EmptyMemory: $Memory;
axiom domain#$Memory($EmptyMemory) == $ConstMemoryDomain(false);
axiom contents#$Memory($EmptyMemory) == $ConstMemoryContent($DefaultValue());

var $abort_flag: bool;
var $abort_code: int;

function {:inline} $process_abort_code(code: int): int {
    code
}

const $EXEC_FAILURE_CODE: int;
axiom $EXEC_FAILURE_CODE == -1;

// TODO(wrwg): currently we map aborts of native functions like those for vectors also to
//   execution failure. This may need to be aligned with what the runtime actually does.

procedure {:inline 1} $ExecFailureAbort() {
    $abort_flag := true;
    $abort_code := $EXEC_FAILURE_CODE;
}

procedure {:inline 1} $InitVerification() {
  // Set abort_flag to false, and havoc abort_code
  $abort_flag := false;
  havoc $abort_code;
}

// ============================================================================================
// Functional APIs

// TODO: unify some of this with instruction procedures to avoid duplication

// Tests whether resource exists.
function {:inline} $ResourceExistsRaw(m: $Memory, args: $TypeValueArray, addr: int): bool {
    domain#$Memory(m)[args, addr]
}
function {:inline} $ResourceExists(m: $Memory, args: $TypeValueArray, addr: $Value): $Value {
    $Boolean($ResourceExistsRaw(m, args, a#$Address(addr)))
}

// Obtains Value of given resource.
function {:inline} $ResourceValue(m: $Memory, args: $TypeValueArray, addr: $Value): $Value {
  contents#$Memory(m)[args, a#$Address(addr)]
}

// Applies a field selection to a Value.
function {:inline} $SelectField(val: $Value, field: $FieldName): $Value {
    $select_vector(val, field)
}

// Updates a field.
function {:inline} $UpdateField(val: $Value, field: $FieldName, new_value: $Value): $Value {
    $update_vector(val, field, new_value)
}


// Dereferences a reference.
function {:inline} $Dereference(ref: $Mutation): $Value {
    v#$Mutation(ref)
}

// ============================================================================================
// Instructions

procedure {:inline 1} $MoveToRaw(m: $Memory, ta: $TypeValueArray, a: int, v: $Value) returns (m': $Memory)
{
    if ($ResourceExistsRaw(m, ta, a)) {
        call $ExecFailureAbort();
        return;
    }
    m' := $Memory(domain#$Memory(m)[ta, a := true], contents#$Memory(m)[ta, a := v]);
}

procedure {:inline 1} $MoveTo(m: $Memory, ta: $TypeValueArray, v: $Value, signer: $Value) returns (m': $Memory)
{
    var address: $Value;
    var a: int;

    call address := $Signer_borrow_address(signer);
    a := a#$Address(address);
    call m' := $MoveToRaw(m, ta, a, v);
}

procedure {:inline 1} $MoveFrom(m: $Memory, address: $Value, ta: $TypeValueArray) returns (m': $Memory, dst: $Value)
free requires is#$Address(address);
{
    var a: int;

    a := a#$Address(address);
    if (!$ResourceExistsRaw(m, ta, a)) {
        call $ExecFailureAbort();
        return;
    }
    dst := contents#$Memory(m)[ta, a];
    m' := $Memory(domain#$Memory(m)[ta, a := false], contents#$Memory(m)[ta, a := $DefaultValue()]);
}

procedure {:inline 1} $BorrowGlobal(m: $Memory, address: $Value, ta: $TypeValueArray) returns (dst: $Mutation)
free requires is#$Address(address);
{
    var a: int;

    a := a#$Address(address);
    if (!$ResourceExistsRaw(m, ta, a)) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Mutation($Global(ta, a), $EmptyPath, contents#$Memory(m)[ta, a]);
}

procedure {:inline 1} $BorrowLoc(l: int, v: $Value) returns (dst: $Mutation)
{
    dst := $Mutation($Local(l), $EmptyPath, v);
}

procedure {:inline 1} $BorrowField(src: $Mutation, f: $FieldName) returns (dst: $Mutation)
{
    var p: $Path;
    var size: int;

    p := p#$Mutation(src);
    size := size#$Path(p);
    p := $Path(p#$Path(p)[size := f], size+1);
    dst := $Mutation(l#$Mutation(src), p, $select_vector(v#$Mutation(src), f));
}

procedure {:inline 1} $GetGlobal(m: $Memory, address: $Value, ta: $TypeValueArray) returns (dst: $Value)
free requires is#$Address(address);
{
    var a: int;
    a := a#$Address(address);
    if (!$ResourceExistsRaw(m, ta, a)) {
        call $ExecFailureAbort();
        return;
    }
    dst := $ResourceValue(m, ta, address);
}

procedure {:inline 1} $GetFieldFromReference(src: $Mutation, f: $FieldName) returns (dst: $Value)
{
    var r: $Mutation;

    call r := $BorrowField(src, f);
    call dst := $ReadRef(r);
}

procedure {:inline 1} $GetFieldFromValue(src: $Value, f: $FieldName) returns (dst: $Value)
{
    dst := $select_vector(src, f);
}

procedure {:inline 1} $WriteRef(to: $Mutation, new_v: $Value) returns (to': $Mutation)
{
    to' := $Mutation(l#$Mutation(to), p#$Mutation(to), new_v);
}

procedure {:inline 1} $ReadRef(from: $Mutation) returns (v: $Value)
{
    v := v#$Mutation(from);
}

procedure {:inline 1} $CopyOrMoveRef(local: $Mutation) returns (dst: $Mutation)
{
    dst := local;
}

procedure {:inline 1} $CopyOrMoveValue(local: $Value) returns (dst: $Value)
{
    dst := local;
}

procedure {:inline 1} $WritebackToGlobal(m: $Memory, src: $Mutation) returns (m': $Memory)
{
    var l: $Location;
    var ta: $TypeValueArray;
    var a: int;
    var v: $Value;

    l := l#$Mutation(src);
    if (is#$Global(l)) {
        ta := ts#$Global(l);
        a := a#$Global(l);
        v := $UpdateValue(p#$Mutation(src), 0, contents#$Memory(m)[ta, a], v#$Mutation(src));
        m' := $Memory(domain#$Memory(m), contents#$Memory(m)[ta, a := v]);
    } else {
        m' := m;
    }
}

procedure {:inline 1} $WritebackToValue(src: $Mutation, idx: int, vdst: $Value) returns (vdst': $Value)
{
    if (l#$Mutation(src) == $Local(idx)) {
        vdst' := $UpdateValue(p#$Mutation(src), 0, vdst, v#$Mutation(src));
    } else {
        vdst' := vdst;
    }
}

procedure {:inline 1} $WritebackToReference(src: $Mutation, dst: $Mutation) returns (dst': $Mutation)
{
    var srcPath, dstPath: $Path;

    srcPath := p#$Mutation(src);
    dstPath := p#$Mutation(dst);
    if (l#$Mutation(dst) == l#$Mutation(src) && size#$Path(dstPath) <= size#$Path(srcPath) && $IsPathPrefix(dstPath, srcPath)) {
        dst' := $Mutation(
                    l#$Mutation(dst),
                    dstPath,
                    $UpdateValue(srcPath, size#$Path(dstPath), v#$Mutation(dst), v#$Mutation(src)));
    } else {
        dst' := dst;
    }
}

procedure {:inline 1} $Splice1(idx1: int, src1: $Mutation, dst: $Mutation) returns (dst': $Mutation) {
    dst' := $Mutation(l#$Mutation(src1), $ConcatPath(p#$Mutation(src1), p#$Mutation(dst)), v#$Mutation(dst));
}

procedure {:inline 1} $CastU8(src: $Value) returns (dst: $Value)
free requires is#$Integer(src);
{
    if (i#$Integer(src) > $MAX_U8) {
        call $ExecFailureAbort();
        return;
    }
    dst := src;
}

procedure {:inline 1} $CastU64(src: $Value) returns (dst: $Value)
free requires is#$Integer(src);
{
    if (i#$Integer(src) > $MAX_U64) {
        call $ExecFailureAbort();
        return;
    }
    dst := src;
}

procedure {:inline 1} $CastU128(src: $Value) returns (dst: $Value)
free requires is#$Integer(src);
{
    if (i#$Integer(src) > $MAX_U128) {
        call $ExecFailureAbort();
        return;
    }
    dst := src;
}

procedure {:inline 1} $AddU8(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU8(src1) && $IsValidU8(src2);
{
    if (i#$Integer(src1) + i#$Integer(src2) > $MAX_U8) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $AddU64(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU64(src1) && $IsValidU64(src2);
{
    if (i#$Integer(src1) + i#$Integer(src2) > $MAX_U64) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $AddU64_unchecked(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU64(src1) && $IsValidU64(src2);
{
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $AddU128(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU128(src1) && $IsValidU128(src2);
{
    if (i#$Integer(src1) + i#$Integer(src2) > $MAX_U128) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $AddU128_unchecked(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU128(src1) && $IsValidU128(src2);
{
    dst := $Integer(i#$Integer(src1) + i#$Integer(src2));
}

procedure {:inline 1} $Sub(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    if (i#$Integer(src1) < i#$Integer(src2)) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) - i#$Integer(src2));
}

// This deals only with narrow special cases. Src2 must be constant
// 32 or 64, which is what we use now.  Obviously, it could be extended
// to src2 == any integer Value from 0..127.
// Left them out for brevity
function $power_of_2(power: $Value): int {
    (var p := i#$Integer(power);
     if p == 8 then 256
     else if p == 16 then 65536
     else if p == 32 then 4294967296
     else if p == 64 then 18446744073709551616
     // Value is undefined, otherwise.
     else -1
     )
}

function $shl(src1: $Value, src2: $Value): $Value {
   (var po2 := $power_of_2(src2);
    $Integer(i#$Integer(src1) * po2)
   )
}

function $shr(src1: $Value, src2: $Value): $Value {
   (var po2 := $power_of_2(src2);
    $Integer(i#$Integer(src1) div po2)
   )
}

// TODO: fix this and $Shr to drop bits on overflow. Requires $Shl8, $Shl64, and $Shl128
procedure {:inline 1} $Shl(src1: $Value, src2: $Value) returns (dst: $Value)
requires is#$Integer(src1) && is#$Integer(src2);
{
    var po2: int;
    po2 := $power_of_2(src2);
    assert po2 >= 1;   // restriction: shift argument must be 8, 16, 32, or 64
    dst := $Integer(i#$Integer(src1) * po2);
}

procedure {:inline 1} $Shr(src1: $Value, src2: $Value) returns (dst: $Value)
requires is#$Integer(src1) && is#$Integer(src2);
{
    var po2: int;
    po2 := $power_of_2(src2);
    assert po2 >= 1;   // restriction: shift argument must be 8, 16, 32, or 64
    dst := $Integer(i#$Integer(src1) div po2);
}

procedure {:inline 1} $MulU8(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU8(src1) && $IsValidU8(src2);
{
    if (i#$Integer(src1) * i#$Integer(src2) > $MAX_U8) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) * i#$Integer(src2));
}

procedure {:inline 1} $MulU64(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU64(src1) && $IsValidU64(src2);
{
    if (i#$Integer(src1) * i#$Integer(src2) > $MAX_U64) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) * i#$Integer(src2));
}

procedure {:inline 1} $MulU128(src1: $Value, src2: $Value) returns (dst: $Value)
free requires $IsValidU128(src1) && $IsValidU128(src2);
{
    if (i#$Integer(src1) * i#$Integer(src2) > $MAX_U128) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) * i#$Integer(src2));
}

procedure {:inline 1} $Div(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    if (i#$Integer(src2) == 0) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) div i#$Integer(src2));
}

procedure {:inline 1} $Mod(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    if (i#$Integer(src2) == 0) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Integer(i#$Integer(src1) mod i#$Integer(src2));
}

procedure {:inline 1} $ArithBinaryUnimplemented(src1: $Value, src2: $Value) returns (dst: $Value);
free requires is#$Integer(src1) && is#$Integer(src2);
ensures is#$Integer(dst);

procedure {:inline 1} $Lt(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) < i#$Integer(src2));
}

procedure {:inline 1} $Gt(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) > i#$Integer(src2));
}

procedure {:inline 1} $Le(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) <= i#$Integer(src2));
}

procedure {:inline 1} $Ge(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Integer(src1) && is#$Integer(src2);
{
    dst := $Boolean(i#$Integer(src1) >= i#$Integer(src2));
}

procedure {:inline 1} $And(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Boolean(src1) && is#$Boolean(src2);
{
    dst := $Boolean(b#$Boolean(src1) && b#$Boolean(src2));
}

procedure {:inline 1} $Or(src1: $Value, src2: $Value) returns (dst: $Value)
free requires is#$Boolean(src1) && is#$Boolean(src2);
{
    dst := $Boolean(b#$Boolean(src1) || b#$Boolean(src2));
}

procedure {:inline 1} $Not(src: $Value) returns (dst: $Value)
free requires is#$Boolean(src);
{
    dst := $Boolean(!b#$Boolean(src));
}

// Pack and Unpack are auto-generated for each type T


// ==================================================================================
// Native Vector Type

function {:inline} $Vector_type_value(tv: $TypeValue): $TypeValue {
    $VectorType(tv)
}



// This is uses the implementation of $ValueArray using integer maps
function {:inline} $Vector_$is_well_formed(v: $Value): bool {
    is#$Vector(v) &&
    (
        var va := v#$Vector(v);
        (
            var l := l#$ValueArray(va);
            0 <= l && l <= $MAX_U64 &&
            (forall x: int :: {v#$ValueArray(va)[x]} x < 0 || x >= l ==> v#$ValueArray(va)[x] == $DefaultValue())
        )
    )
}



procedure {:inline 1} $Vector_empty(ta: $TypeValue) returns (v: $Value) {
    v := $mk_vector();
}

function {:inline 1} $Vector_$empty(ta: $TypeValue): $Value {
    $mk_vector()
}

procedure {:inline 1} $Vector_is_empty(ta: $TypeValue, v: $Value) returns (b: $Value) {
    assume is#$Vector(v);
    b := $Boolean($vlen(v) == 0);
}

procedure {:inline 1} $Vector_push_back(ta: $TypeValue, v: $Value, val: $Value) returns (v': $Value) {
    assume is#$Vector(v);
    v' := $push_back_vector(v, val);
}

function {:inline 1} $Vector_$push_back(ta: $TypeValue, v: $Value, val: $Value): $Value {
    $push_back_vector(v, val)
}

procedure {:inline 1} $Vector_pop_back(ta: $TypeValue, v: $Value) returns (e: $Value, v': $Value) {
    var len: int;
    assume is#$Vector(v);
    len := $vlen(v);
    if (len == 0) {
        call $ExecFailureAbort();
        return;
    }
    e := $select_vector(v, len-1);
    v' := $pop_back_vector(v);
}

function {:inline 1} $Vector_$pop_back(ta: $TypeValue, v: $Value): $Value {
    $select_vector(v, $vlen(v)-1)
}

procedure {:inline 1} $Vector_append(ta: $TypeValue, v: $Value, other: $Value) returns (v': $Value) {
    assume is#$Vector(v);
    assume is#$Vector(other);
    v' := $append_vector(v, other);
}

procedure {:inline 1} $Vector_reverse(ta: $TypeValue, v: $Value) returns (v': $Value) {
    assume is#$Vector(v);
    v' := $reverse_vector(v);
}

procedure {:inline 1} $Vector_length(ta: $TypeValue, v: $Value) returns (l: $Value) {
    assume is#$Vector(v);
    l := $Integer($vlen(v));
}

function {:inline 1} $Vector_$length(ta: $TypeValue, v: $Value): $Value {
    $Integer($vlen(v))
}

procedure {:inline 1} $Vector_borrow(ta: $TypeValue, v: $Value, i: $Value) returns (dst: $Value) {
    var i_ind: int;

    assume is#$Vector(v);
    assume is#$Integer(i);
    i_ind := i#$Integer(i);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        call $ExecFailureAbort();
        return;
    }
    dst := $select_vector(v, i_ind);
}

function {:inline 1} $Vector_$borrow(ta: $TypeValue, v: $Value, i: $Value): $Value {
    $select_vector(v, i#$Integer(i))
}

procedure {:inline 1} $Vector_borrow_mut(ta: $TypeValue, v: $Value, index: $Value) returns (dst: $Mutation, v': $Value)
free requires is#$Integer(index);
{
    var i_ind: int;

    i_ind := i#$Integer(index);
    assume is#$Vector(v);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        call $ExecFailureAbort();
        return;
    }
    dst := $Mutation($Local(0), $Path(p#$Path($EmptyPath)[0 := i_ind], 1), $select_vector(v, i_ind));
    v' := v;
}

function {:inline 1} $Vector_$borrow_mut(ta: $TypeValue, v: $Value, i: $Value): $Value {
    $select_vector(v, i#$Integer(i))
}

procedure {:inline 1} $Vector_destroy_empty(ta: $TypeValue, v: $Value) {
    if ($vlen(v) != 0) {
      call $ExecFailureAbort();
    }
}

procedure {:inline 1} $Vector_swap(ta: $TypeValue, v: $Value, i: $Value, j: $Value) returns (v': $Value)
free requires is#$Integer(i) && is#$Integer(j);
{
    var i_ind: int;
    var j_ind: int;
    assume is#$Vector(v);
    i_ind := i#$Integer(i);
    j_ind := i#$Integer(j);
    if (i_ind >= $vlen(v) || j_ind >= $vlen(v) || i_ind < 0 || j_ind < 0) {
        call $ExecFailureAbort();
        return;
    }
    v' := $swap_vector(v, i_ind, j_ind);
}

function {:inline 1} $Vector_$swap(ta: $TypeValue, v: $Value, i: $Value, j: $Value): $Value {
    $swap_vector(v, i#$Integer(i), i#$Integer(j))
}

procedure {:inline 1} $Vector_remove(ta: $TypeValue, v: $Value, i: $Value) returns (e: $Value, v': $Value)
free requires is#$Integer(i);
{
    var i_ind: int;

    assume is#$Vector(v);
    i_ind := i#$Integer(i);
    if (i_ind < 0 || i_ind >= $vlen(v)) {
        call $ExecFailureAbort();
        return;
    }
    e := $select_vector(v, i_ind);
    v' := $remove_vector(v, i_ind);
}

procedure {:inline 1} $Vector_swap_remove(ta: $TypeValue, v: $Value, i: $Value) returns (e: $Value, v': $Value)
free requires is#$Integer(i);
{
    var i_ind: int;
    var len: int;

    assume is#$Vector(v);
    i_ind := i#$Integer(i);
    len := $vlen(v);
    if (i_ind < 0 || i_ind >= len) {
        call $ExecFailureAbort();
        return;
    }
    e := $select_vector(v, i_ind);
    v' := $pop_back_vector($swap_vector(v, i_ind, len-1));
}

procedure {:inline 1} $Vector_contains(ta: $TypeValue, v: $Value, e: $Value) returns (res: $Value)  {
    assume is#$Vector(v);
    res := $Boolean($contains_vector(v, e));
}

// FIXME: This procedure sometimes (not always) make the test (performance_200511) very slow (> 10 mins) or hang
// although this is not used in the test script (performance_200511). The test finishes in 20 secs when it works fine.
procedure {:inline 1} $Vector_index_of(ta: $TypeValue, v: $Value, e: $Value) returns (res1: $Value, res2: $Value);
requires is#$Vector(v);
ensures is#$Boolean(res1);
ensures is#$Integer(res2);
ensures 0 <= i#$Integer(res2) && i#$Integer(res2) < $vlen(v);
ensures res1 == $Boolean($contains_vector(v, e));
ensures b#$Boolean(res1) ==> $IsEqual($select_vector(v,i#$Integer(res2)), e);
ensures b#$Boolean(res1) ==> (forall i:int :: 0<=i && i<i#$Integer(res2) ==> !$IsEqual($select_vector(v,i), e));
ensures !b#$Boolean(res1) ==> i#$Integer(res2) == 0;

// FIXME: This alternative definition has the same issue as the other one above.
// TODO: Delete this when unnecessary
//procedure {:inline 1} $Vector_index_of(ta: $TypeValue, v: $Value, e: $Value) returns (res1: $Value, res2: $Value) {
//    var b: bool;
//    var i: int;
//    assume is#$Vector(v);
//    b := $contains_vector(v, e);
//    if (b) {
//        havoc i;
//        assume 0 <= i && i < $vlen(v);
//        assume $IsEqual($select_vector(v,i), e);
//        assume (forall j:int :: 0<=j && j<i ==> !$IsEqual($select_vector(v,j), e));
//    }
//    else {
//        i := 0;
//    }
//    res1 := $Boolean(b);
//    res2 := $Integer(i);
//}

// ==================================================================================
// Native hash

// Hash is modeled as an otherwise uninterpreted injection.
// In truth, it is not an injection since the domain has greater cardinality
// (arbitrary length vectors) than the co-domain (vectors of length 32).  But it is
// common to assume in code there are no hash collisions in practice.  Fortunately,
// Boogie is not smart enough to recognized that there is an inconsistency.
// FIXME: If we were using a reliable extensional theory of arrays, and if we could use ==
// instead of $IsEqual, we might be able to avoid so many quantified formulas by
// using a sha2_inverse function in the ensures conditions of Hash_sha2_256 to
// assert that sha2/3 are injections without using global quantified axioms.


function {:inline} $Hash_sha2(val: $Value): $Value {
    $Hash_sha2_core(val)
}

function $Hash_sha2_core(val: $Value): $Value;

// This says that Hash_sha2 respects isEquals (this would be automatic if we had an
// extensional theory of arrays and used ==, which has the substitution property
// for functions).
axiom (forall v1,v2: $Value :: $Vector_$is_well_formed(v1) && $Vector_$is_well_formed(v2)
       && $IsEqual(v1, v2) ==> $IsEqual($Hash_sha2_core(v1), $Hash_sha2_core(v2)));

// This says that Hash_sha2 is an injection
axiom (forall v1,v2: $Value :: $Vector_$is_well_formed(v1) && $Vector_$is_well_formed(v2)
        && $IsEqual($Hash_sha2_core(v1), $Hash_sha2_core(v2)) ==> $IsEqual(v1, v2));

// This procedure has no body. We want Boogie to just use its requires
// and ensures properties when verifying code that calls it.
procedure $Hash_sha2_256(val: $Value) returns (res: $Value);
// It will still work without this, but this helps verifier find more reasonable counterexamples.
free requires $IsValidU8Vector(val);
ensures res == $Hash_sha2_core(val);     // returns Hash_sha2 Value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures $vlen(res) == 32;               // result is 32 bytes.

// Spec version of Move native function.
function {:inline} $Hash_$sha2_256(val: $Value): $Value {
    $Hash_sha2_core(val)
}

// similarly for Hash_sha3
function {:inline} $Hash_sha3(val: $Value): $Value {
    $Hash_sha3_core(val)
}
function $Hash_sha3_core(val: $Value): $Value;

axiom (forall v1,v2: $Value :: $Vector_$is_well_formed(v1) && $Vector_$is_well_formed(v2)
       && $IsEqual(v1, v2) ==> $IsEqual($Hash_sha3_core(v1), $Hash_sha3_core(v2)));

axiom (forall v1,v2: $Value :: $Vector_$is_well_formed(v1) && $Vector_$is_well_formed(v2)
        && $IsEqual($Hash_sha3_core(v1), $Hash_sha3_core(v2)) ==> $IsEqual(v1, v2));

procedure $Hash_sha3_256(val: $Value) returns (res: $Value);
ensures res == $Hash_sha3_core(val);     // returns Hash_sha3 Value
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.
ensures $vlen(res) == 32;               // result is 32 bytes.

// Spec version of Move native function.
function {:inline} $Hash_$sha3_256(val: $Value): $Value {
    $Hash_sha3_core(val)
}

// ==================================================================================
// Native libra_account

procedure {:inline 1} $Account_create_signer(
  addr: $Value
) returns (signer: $Value) {
    // A signer is currently identical to an address.
    signer := addr;
}

procedure {:inline 1} $Account_destroy_signer(
  signer: $Value
) {
  return;
}

procedure {:inline 1} $Account_write_to_event_store(ta: $TypeValue, guid: $Value, count: $Value, msg: $Value) {
    // TODO: this is used in old library sources, remove it once those sources are not longer used in tests.
    // This function is modeled as a no-op because the actual side effect of this native function is not observable from the Move side.
}

procedure {:inline 1} $Event_write_to_event_store(ta: $TypeValue, guid: $Value, count: $Value, msg: $Value) {
    // This function is modeled as a no-op because the actual side effect of this native function is not observable from the Move side.
}

// ==================================================================================
// Native Signer

procedure {:inline 1} $Signer_borrow_address(signer: $Value) returns (res: $Value)
    free requires is#$Address(signer);
{
    res := signer;
}

// ==================================================================================
// Native signature

// Signature related functionality is handled via uninterpreted functions. This is sound
// currently because we verify every code path based on signature verification with
// an arbitrary interpretation.

function $Signature_$ed25519_validate_pubkey(public_key: $Value): $Value;
function $Signature_$ed25519_verify(signature: $Value, public_key: $Value, message: $Value): $Value;

axiom (forall public_key: $Value ::
        is#$Boolean($Signature_$ed25519_validate_pubkey(public_key)));

axiom (forall signature, public_key, message: $Value ::
        is#$Boolean($Signature_$ed25519_verify(signature, public_key, message)));


procedure {:inline 1} $Signature_ed25519_validate_pubkey(public_key: $Value) returns (res: $Value) {
    res := $Signature_$ed25519_validate_pubkey(public_key);
}

procedure {:inline 1} $Signature_ed25519_verify(
        signature: $Value, public_key: $Value, message: $Value) returns (res: $Value) {
    res := $Signature_$ed25519_verify(signature, public_key, message);
}

// ==================================================================================
// Native SCS::serialize

// native define serialize<MoveValue>(v: &MoveValue): vector<u8>;

// Serialize is modeled as an uninterpreted function, with an additional
// axiom to say it's an injection.

function {:inline} $SCS_serialize(ta: $TypeValue, v: $Value): $Value {
    $SCS_serialize_core(v)
}

function $SCS_serialize_core(v: $Value): $Value;
function $SCS_serialize_core_inv(v: $Value): $Value;
// Needed only because IsEqual(v1, v2) is weaker than v1 == v2 in case there is a vector nested inside v1 or v2.
axiom (forall v1, v2: $Value :: $IsEqual(v1, v2) ==> $SCS_serialize_core(v1) == $SCS_serialize_core(v2));
// Injectivity
axiom (forall v: $Value :: $SCS_serialize_core_inv($SCS_serialize_core(v)) == v);

// This says that serialize returns a non-empty vec<u8>

axiom (forall v: $Value :: ( var r := $SCS_serialize_core(v); $IsValidU8Vector(r) && $vlen(r) > 0 ));


// Serialized addresses should have the same length
const $serialized_address_len: int;
axiom (forall v: $Value :: (var r := $SCS_serialize_core(v); is#$Address(v) ==> $vlen(r) == $serialized_address_len));

procedure $SCS_to_bytes(ta: $TypeValue, v: $Value) returns (res: $Value);
ensures res == $SCS_serialize(ta, v);
ensures $IsValidU8Vector(res);    // result is a legal vector of U8s.

function {:inline} $SCS_$to_bytes(ta: $TypeValue, v: $Value): $Value {
    $SCS_serialize_core(v)
}

procedure $SCS_to_address(v: $Value) returns (res: $Value);

// ==================================================================================
// Native Signer::spec_address_of

function {:inline} $Signer_spec_address_of(signer: $Value): $Value
{
    // A signer is currently identical to an address.
    signer
}

function {:inline} $Signer_$borrow_address(signer: $Value): $Value
{
    // A signer is currently identical to an address.
    signer
}

// ==================================================================================
// Mocked out Event module

procedure {:inline 1} $Event_new_event_handle(t: $TypeValue, signer: $Value) returns (res: $Value) {
}

procedure {:inline 1} $Event_publish_generator(account: $Value) {
}

procedure {:inline 1} $Event_emit_event(t: $TypeValue, handler: $Value, msg: $Value) returns (res: $Value) {
    res := handler;
}

procedure {:inline 1} $Event_destroy_handle(t: $TypeValue, handler: $Value) {
}

procedure $Token_name_of(t_E: $TypeValue) returns (res1: $Value, res2: $Value, res3: $Value);
ensures $IsValidU8Vector(res2);
ensures $IsValidU8Vector(res3);

procedure $Debug_print(t_T: $TypeValue, x: $Value);
procedure $Debug_print_stack_trace();



// ** spec vars of module Signer



// ** spec funs of module Signer

function {:inline} $Signer_$address_of(s: $Value): $Value {
    $Signer_$borrow_address(s)
}



// ** structs of module Signer



// ** functions of module Signer

procedure {:inline 1} $Signer_address_of_$def(s: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $AddressType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := s;
      assume {:print "$track_local(9,489,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(s)
    call $t1 := $CopyOrMoveValue(s);

    // $t2 := Signer::borrow_address($t1)
    call $t2 := $Signer_borrow_address($t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(9,406):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t2
    $ret0 := $t2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(9,542,3):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Signer_address_of_$direct_inter(s: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Signer_spec_address_of(s)))));
ensures is#$Address($ret0);

procedure {:inline 1} $Signer_address_of_$direct_intra(s: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Signer_spec_address_of(s)))));
ensures is#$Address($ret0);

procedure {:inline 1} $Signer_address_of(s: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Signer_spec_address_of(s)))));
ensures is#$Address($ret0);



// ** spec vars of module Vector



// ** spec funs of module Vector

function {:inline} $Vector_spec_singleton($tv0: $TypeValue, e: $Value): $Value {
    $single_vector(e)
}

function {:inline} $Vector_spec_contains($tv0: $TypeValue, v: $Value, e: $Value): $Value {
    $Boolean((var $range_1 := v; (exists $i_0: int :: $InVectorRange($range_1, $i_0) && (var x := $select_vector($range_1, $i_0); b#$Boolean($Boolean($IsEqual(x, e)))))))
}

function {:inline} $Vector_eq_push_back($tv0: $TypeValue, v1: $Value, v2: $Value, e: $Value): $Value {
    $Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($vlen_value(v1), $Integer(i#$Integer($vlen_value(v2)) + i#$Integer($Integer(1)))))) && b#$Boolean($Boolean($IsEqual($select_vector_by_value(v1, $Integer(i#$Integer($vlen_value(v1)) - i#$Integer($Integer(1)))), e))))) && b#$Boolean($Boolean($IsEqual($slice_vector(v1, $Range($Integer(0), $Integer(i#$Integer($vlen_value(v1)) - i#$Integer($Integer(1))))), $slice_vector(v2, $Range($Integer(0), $vlen_value(v2)))))))
}

function {:inline} $Vector_eq_append($tv0: $TypeValue, v: $Value, v1: $Value, v2: $Value): $Value {
    $Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($vlen_value(v), $Integer(i#$Integer($vlen_value(v1)) + i#$Integer($vlen_value(v2)))))) && b#$Boolean($Boolean($IsEqual($slice_vector(v, $Range($Integer(0), $vlen_value(v1))), v1))))) && b#$Boolean($Boolean($IsEqual($slice_vector(v, $Range($vlen_value(v1), $vlen_value(v))), v2))))
}

function {:inline} $Vector_eq_pop_front($tv0: $TypeValue, v1: $Value, v2: $Value): $Value {
    $Boolean(b#$Boolean($Boolean($IsEqual($Integer(i#$Integer($vlen_value(v1)) + i#$Integer($Integer(1))), $vlen_value(v2)))) && b#$Boolean($Boolean($IsEqual(v1, $slice_vector(v2, $Range($Integer(1), $vlen_value(v2)))))))
}



// ** structs of module Vector



// ** functions of module Vector

procedure {:inline 1} $Vector_singleton_$def($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var v: $Value; // $Vector_type_value($tv0)
    var $t2: $Value; // $tv0
    var $t3: $Mutation; // ReferenceType($Vector_type_value($tv0))
    var $t4: $Value; // $Vector_type_value($tv0)
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := e;
      assume {:print "$track_local(11,1332,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(e)
    call $t2 := $CopyOrMoveValue(e);

    // v := Vector::empty<#0>()
    call v := $Vector_empty($tv0);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,262):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t3 := borrow_local(v)
    call $t3 := $BorrowLoc(1, v);

    // unpack_ref($t3)

    // $t4 := read_ref($t3)
    call $t4 := $ReadRef($t3);

    // $t4 := Vector::push_back<#0>($t4, $t2)
    call $t4 := $Vector_push_back($tv0, $t4, $t2);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,625):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t3, $t4)
    call $t3 := $WriteRef($t3, $t4);

    // pack_ref($t3)

    // write_back[LocalRoot(v)]($t3)
    call v := $WritebackToValue($t3, 1, v);

    // return v
    $ret0 := v;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(11,1456,5):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Vector_singleton_$direct_inter($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
{
    call $ret0 := $Vector_singleton_$def($tv0, e);
}


procedure {:inline 1} $Vector_singleton_$direct_intra($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
{
    call $ret0 := $Vector_singleton_$def($tv0, e);
}


procedure {:inline 1} $Vector_singleton($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
{
    call $ret0 := $Vector_singleton_$def($tv0, e);
}


procedure {:inline 1} $Vector_split_$def($tv0: $TypeValue, v: $Value, sub_len: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var i: $Value; // $IntegerType()
    var index: $Value; // $IntegerType()
    var index#462: $Value; // $IntegerType()
    var j: $Value; // $IntegerType()
    var len: $Value; // $IntegerType()
    var rem: $Value; // $IntegerType()
    var result: $Value; // $Vector_type_value($Vector_type_value($tv0))
    var sub: $Value; // $Vector_type_value($tv0)
    var sub#461: $Value; // $Vector_type_value($tv0)
    var $t11: $Value; // $Vector_type_value($tv0)
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $BooleanType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $BooleanType()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $BooleanType()
    var $t24: $Value; // $IntegerType()
    var $t25: $Mutation; // ReferenceType($Vector_type_value($tv0))
    var $t26: $Value; // $tv0
    var $t27: $Value; // $Vector_type_value($tv0)
    var $t28: $Value; // $IntegerType()
    var $t29: $Mutation; // ReferenceType($Vector_type_value($Vector_type_value($tv0)))
    var $t30: $Value; // $Vector_type_value($Vector_type_value($tv0))
    var $t31: $Value; // $IntegerType()
    var $t32: $Value; // $IntegerType()
    var $t33: $Value; // $BooleanType()
    var $t34: $Value; // $IntegerType()
    var $t35: $Value; // $IntegerType()
    var $t36: $Value; // $BooleanType()
    var $t37: $Mutation; // ReferenceType($Vector_type_value($tv0))
    var $t38: $Value; // $tv0
    var $t39: $Value; // $IntegerType()
    var $t40: $Mutation; // ReferenceType($Vector_type_value($Vector_type_value($tv0)))
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := v;
      assume {:print "$track_local(11,4174,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := sub_len;
      assume {:print "$track_local(11,4174,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t11 := move(v)
    call $t11 := $CopyOrMoveValue(v);

    // $t12 := move(sub_len)
    call $t12 := $CopyOrMoveValue(sub_len);

    // result := Vector::empty<vector<#0>>()
    call result := $Vector_empty($Vector_type_value($tv0));
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,262):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t13 := Vector::length<#0>($t11)
    call $t13 := $Vector_length($tv0, $t11);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,360):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // len := /($t13, $t12)
    call len := $Div($t13, $t12);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,4347):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := len;
      assume {:print "$track_local(11,4347,6):", $trace_temp} true;
    }

    // $t14 := 0
    $t14 := $Integer(0);

    // rem := $t14
    call rem := $CopyOrMoveValue($t14);
    if (true) {
     $trace_temp := rem;
      assume {:print "$track_local(11,4371,7):", $trace_temp} true;
    }

    // $t15 := *(len, $t12)
    call $t15 := $MulU64(len, $t12);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,4396):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t16 := Vector::length<#0>($t11)
    call $t16 := $Vector_length($tv0, $t11);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,360):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t17 := <($t15, $t16)
    call $t17 := $Lt($t15, $t16);

    // if ($t17) goto L0 else goto L1
    if (b#$Boolean($t17)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t18 := Vector::length<#0>($t11)
    call $t18 := $Vector_length($tv0, $t11);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,360):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t19 := *(len, $t12)
    call $t19 := $MulU64(len, $t12);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,4455):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // rem := -($t18, $t19)
    call rem := $Sub($t18, $t19);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,4449):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := rem;
      assume {:print "$track_local(11,4449,7):", $trace_temp} true;
    }

    // goto L2
    goto L2;

    // L2:
L2:

    // $t20 := 0
    $t20 := $Integer(0);

    // i := $t20
    call i := $CopyOrMoveValue($t20);
    if (true) {
     $trace_temp := i;
      assume {:print "$track_local(11,4490,2):", $trace_temp} true;
    }

    // goto L10
    goto L10;

    // L10:
L10:
assume !$abort_flag;
assume $IsValidU64(i);
assume $IsValidU64(index);
assume $IsValidU64(j);
assume $Vector_$is_well_formed(result) && (forall $$0: int :: {$select_vector(result,$$0)} $$0 >= 0 && $$0 < $vlen(result) ==> $Vector_$is_well_formed($select_vector(result,$$0)));
assume $Vector_$is_well_formed(sub);
assume is#$Boolean($t21);
assume $IsValidU64($t22);
assume is#$Boolean($t23);
assume $IsValidU64($t24);
assume $Vector_$is_well_formed($Dereference($t25));
assume $Vector_$is_well_formed($t27);
assume $IsValidU64($t28);
assume $Vector_$is_well_formed($Dereference($t29)) && (forall $$1: int :: {$select_vector($Dereference($t29),$$1)} $$1 >= 0 && $$1 < $vlen($Dereference($t29)) ==> $Vector_$is_well_formed($select_vector($Dereference($t29),$$1)));
assume $Vector_$is_well_formed($t30) && (forall $$0: int :: {$select_vector($t30,$$0)} $$0 >= 0 && $$0 < $vlen($t30) ==> $Vector_$is_well_formed($select_vector($t30,$$0)));
assume $IsValidU64($t31);

    // $t21 := <(i, len)
    call $t21 := $Lt(i, len);

    // if ($t21) goto L3 else goto L4
    if (b#$Boolean($t21)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L5
    goto L5;

    // L3:
L3:

    // sub := Vector::empty<#0>()
    call sub := $Vector_empty($tv0);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,262):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t22 := 0
    $t22 := $Integer(0);

    // j := $t22
    call j := $CopyOrMoveValue($t22);
    if (true) {
     $trace_temp := j;
      assume {:print "$track_local(11,4579,5):", $trace_temp} true;
    }

    // goto L9
    goto L9;

    // L9:
L9:
assume !$abort_flag;
assume $IsValidU64(index);
assume $IsValidU64(j);
assume $Vector_$is_well_formed(sub);
assume is#$Boolean($t23);
assume $IsValidU64($t24);
assume $Vector_$is_well_formed($Dereference($t25));
assume $Vector_$is_well_formed($t27);
assume $IsValidU64($t28);

    // $t23 := <(j, $t12)
    call $t23 := $Lt(j, $t12);

    // if ($t23) goto L6 else goto L7
    if (b#$Boolean($t23)) { goto L6; } else { goto L7; }

    // L7:
L7:

    // goto L8
    goto L8;

    // L6:
L6:

    // $t24 := *($t12, i)
    call $t24 := $MulU64($t12, i);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,4656):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // index := +($t24, j)
    call index := $AddU64($t24, j);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,4660):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := index;
      assume {:print "$track_local(11,4660,3):", $trace_temp} true;
    }

    // $t25 := borrow_local(sub)
    call $t25 := $BorrowLoc(9, sub);

    // unpack_ref($t25)

    // $t26 := Vector::borrow<#0>($t11, index)
    call $t26 := $Vector_borrow($tv0, $t11, index);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,498):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t27 := read_ref($t25)
    call $t27 := $ReadRef($t25);

    // $t27 := Vector::push_back<#0>($t27, $t26)
    call $t27 := $Vector_push_back($tv0, $t27, $t26);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,625):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t25, $t27)
    call $t25 := $WriteRef($t25, $t27);

    // pack_ref($t25)

    // write_back[LocalRoot(sub)]($t25)
    call sub := $WritebackToValue($t25, 9, sub);

    // $t28 := 1
    $t28 := $Integer(1);

    // j := +(j, $t28)
    call j := $AddU64(j, $t28);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,4743):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := j;
      assume {:print "$track_local(11,4743,5):", $trace_temp} true;
    }

    // goto L9
    goto L9;

    // L8:
L8:

    // $t29 := borrow_local(result)
    call $t29 := $BorrowLoc(8, result);

    // unpack_ref($t29)

    // $t30 := read_ref($t29)
    call $t30 := $ReadRef($t29);

    // $t30 := Vector::push_back<vector<#0>>($t30, sub)
    call $t30 := $Vector_push_back($Vector_type_value($tv0), $t30, sub);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,625):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t29, $t30)
    call $t29 := $WriteRef($t29, $t30);

    // pack_ref($t29)

    // write_back[LocalRoot(result)]($t29)
    call result := $WritebackToValue($t29, 8, result);

    // $t31 := 1
    $t31 := $Integer(1);

    // i := +(i, $t31)
    call i := $AddU64(i, $t31);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,4839):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := i;
      assume {:print "$track_local(11,4839,2):", $trace_temp} true;
    }

    // goto L10
    goto L10;

    // L5:
L5:

    // $t32 := 0
    $t32 := $Integer(0);

    // $t33 := >(rem, $t32)
    call $t33 := $Gt(rem, $t32);

    // if ($t33) goto L11 else goto L12
    if (b#$Boolean($t33)) { goto L11; } else { goto L12; }

    // L12:
L12:

    // goto L13
    goto L13;

    // L11:
L11:

    // sub#461 := Vector::empty<#0>()
    call sub#461 := $Vector_empty($tv0);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,262):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t34 := Vector::length<#0>($t11)
    call $t34 := $Vector_length($tv0, $t11);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,360):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // index#462 := -($t34, rem)
    call index#462 := $Sub($t34, rem);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,4953):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := index#462;
      assume {:print "$track_local(11,4953,4):", $trace_temp} true;
    }

    // goto L17
    goto L17;

    // L17:
L17:
assume !$abort_flag;
assume $IsValidU64(index#462);
assume $Vector_$is_well_formed(sub#461);
assume $Vector_$is_well_formed($t27);
assume $IsValidU64($t35);
assume is#$Boolean($t36);
assume $Vector_$is_well_formed($Dereference($t37));
assume $IsValidU64($t39);

    // $t35 := Vector::length<#0>($t11)
    call $t35 := $Vector_length($tv0, $t11);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,360):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t36 := <(index#462, $t35)
    call $t36 := $Lt(index#462, $t35);

    // if ($t36) goto L14 else goto L15
    if (b#$Boolean($t36)) { goto L14; } else { goto L15; }

    // L15:
L15:

    // goto L16
    goto L16;

    // L14:
L14:

    // $t37 := borrow_local(sub#461)
    call $t37 := $BorrowLoc(10, sub#461);

    // unpack_ref($t37)

    // $t38 := Vector::borrow<#0>($t11, index#462)
    call $t38 := $Vector_borrow($tv0, $t11, index#462);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,498):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t27 := read_ref($t37)
    call $t27 := $ReadRef($t37);

    // $t27 := Vector::push_back<#0>($t27, $t38)
    call $t27 := $Vector_push_back($tv0, $t27, $t38);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,625):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t37, $t27)
    call $t37 := $WriteRef($t37, $t27);

    // pack_ref($t37)

    // write_back[LocalRoot(sub#461)]($t37)
    call sub#461 := $WritebackToValue($t37, 10, sub#461);

    // $t39 := 1
    $t39 := $Integer(1);

    // index#462 := +(index#462, $t39)
    call index#462 := $AddU64(index#462, $t39);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,5086):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := index#462;
      assume {:print "$track_local(11,5086,4):", $trace_temp} true;
    }

    // goto L17
    goto L17;

    // L16:
L16:

    // destroy($t11)

    // $t40 := borrow_local(result)
    call $t40 := $BorrowLoc(8, result);

    // unpack_ref($t40)

    // $t30 := read_ref($t40)
    call $t30 := $ReadRef($t40);

    // $t30 := Vector::push_back<vector<#0>>($t30, sub#461)
    call $t30 := $Vector_push_back($Vector_type_value($tv0), $t30, sub#461);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(11,625):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t40, $t30)
    call $t40 := $WriteRef($t40, $t30);

    // pack_ref($t40)

    // write_back[LocalRoot(result)]($t40)
    call result := $WritebackToValue($t40, 8, result);

    // goto L18
    goto L18;

    // L13:
L13:

    // destroy($t11)

    // goto L18
    goto L18;

    // L18:
L18:

    // return result
    $ret0 := result;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(11,5183,41):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Vector_split_$direct_inter($tv0: $TypeValue, v: $Value, sub_len: $Value) returns ($ret0: $Value)
{
    assume $Vector_$is_well_formed(v);

    assume $IsValidU64(sub_len);

    call $ret0 := $Vector_split_$def($tv0, v, sub_len);
}


procedure {:inline 1} $Vector_split_$direct_intra($tv0: $TypeValue, v: $Value, sub_len: $Value) returns ($ret0: $Value)
{
    assume $Vector_$is_well_formed(v);

    assume $IsValidU64(sub_len);

    call $ret0 := $Vector_split_$def($tv0, v, sub_len);
}


procedure {:inline 1} $Vector_split($tv0: $TypeValue, v: $Value, sub_len: $Value) returns ($ret0: $Value)
{
    assume $Vector_$is_well_formed(v);

    assume $IsValidU64(sub_len);

    call $ret0 := $Vector_split_$def($tv0, v, sub_len);
}




// ** spec vars of module Option



// ** spec funs of module Option

function {:inline} $Option_spec_none($tv0: $TypeValue): $Value {
    $Vector($ExtendValueArray($EmptyValueArray(), $mk_vector()))
}

function {:inline} $Option_spec_some($tv0: $TypeValue, e: $Value): $Value {
    $Vector($ExtendValueArray($EmptyValueArray(), $Vector_spec_singleton($tv0, e)))
}

function {:inline} $Option_spec_is_none($tv0: $TypeValue, t: $Value): $Value {
    $Boolean($IsEqual($vlen_value($SelectField(t, $Option_Option_vec)), $Integer(0)))
}

function {:inline} $Option_spec_is_some($tv0: $TypeValue, t: $Value): $Value {
    $Boolean(!b#$Boolean($Option_spec_is_none($tv0, t)))
}

function {:inline} $Option_spec_contains($tv0: $TypeValue, t: $Value, e: $Value): $Value {
    $Boolean(b#$Boolean($Option_spec_is_some($tv0, t)) && b#$Boolean($Boolean($IsEqual($Option_spec_get($tv0, t), e))))
}

function {:inline} $Option_spec_get($tv0: $TypeValue, t: $Value): $Value {
    $select_vector_by_value($SelectField(t, $Option_Option_vec), $Integer(0))
}



// ** structs of module Option

const unique $Option_Option: $TypeName;
const $Option_Option_vec: $FieldName;
axiom $Option_Option_vec == 0;
function $Option_Option_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Option_Option, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1))
}
var $Option_Option_$memory: $Memory;
var $Option_Option_$memory_$old: $Memory;
function {:inline} $Option_Option_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 1
      && $Vector_$is_well_formed($SelectField($this, $Option_Option_vec))
}
function {:inline} $Option_Option_$invariant_holds($this: $Value): bool {
    b#$Boolean($Boolean(i#$Integer($vlen_value($SelectField($this, $Option_Option_vec))) <= i#$Integer($Integer(1))))
}

function {:inline} $Option_Option_$is_well_formed($this: $Value): bool {
    $Option_Option_$is_well_typed($this) && $Option_Option_$invariant_holds($this)}

procedure {:inline 1} $Option_Option_$unpack_ref_deep($tv0: $TypeValue, $before: $Value) {
    assume $Option_Option_$invariant_holds($before);
}

procedure {:inline 1} $Option_Option_$unpack_ref($tv0: $TypeValue, $before: $Value) {
    assume $Option_Option_$invariant_holds($before);
}

procedure {:inline 1} $Option_Option_$pack_ref_deep($tv0: $TypeValue, $after: $Value) {
    assert b#$Boolean($Boolean(i#$Integer($vlen_value($SelectField($after, $Option_Option_vec))) <= i#$Integer($Integer(1))));
}

procedure {:inline 1} $Option_Option_$pack_ref($tv0: $TypeValue, $after: $Value) {
    assert b#$Boolean($Boolean(i#$Integer($vlen_value($SelectField($after, $Option_Option_vec))) <= i#$Integer($Integer(1))));
}

procedure {:inline 1} $Option_Option_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, vec: $Value) returns ($struct: $Value)
{
    assume $Vector_$is_well_formed(vec);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := vec], 1));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
    assert b#$Boolean($Boolean(i#$Integer($vlen_value($SelectField($struct, $Option_Option_vec))) <= i#$Integer($Integer(1))));
}

procedure {:inline 1} $Option_Option_unpack($tv0: $TypeValue, $struct: $Value) returns (vec: $Value)
{
    assume is#$Vector($struct);
    vec := $SelectField($struct, $Option_Option_vec);
    assume $Vector_$is_well_formed(vec);
}



// ** functions of module Option

procedure {:inline 1} $Option_borrow_$def($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $Option_Option_type_value($tv0)
    var $t2: $Value; // $Vector_type_value($tv0)
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $tv0
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,2981,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(t)
    call $t1 := $CopyOrMoveValue(t);

    // $t2 := get_field<Option::Option<#0>>.vec($t1)
    call $t2 := $GetFieldFromValue($t1, $Option_Option_vec);

    // $t3 := 0
    $t3 := $Integer(0);

    // $t4 := Vector::borrow<#0>($t2, $t3)
    call $t4 := $Vector_borrow($tv0, $t2, $t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,3057):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t4
    $ret0 := $t4;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,3049,5):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Option_borrow_$direct_inter($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, t)))));

procedure {:inline 1} $Option_borrow_$direct_intra($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, t)))));

procedure {:inline 1} $Option_borrow($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, t)))));

procedure {:inline 1} $Option_borrow_mut_$def($tv0: $TypeValue, t: $Value) returns ($ret0: $Mutation, $ret1: $Value)
{
    // declare local variables
    var $t1: $Value; // $Option_Option_type_value($tv0)
    var $t2: $Mutation; // ReferenceType($Option_Option_type_value($tv0))
    var $t3: $Mutation; // ReferenceType($Vector_type_value($tv0))
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $Vector_type_value($tv0)
    var $t6: $Mutation; // ReferenceType($tv0)
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,5461,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(t)
    call $t1 := $CopyOrMoveValue(t);

    // $t2 := borrow_local($t1)
    call $t2 := $BorrowLoc(1, $t1);

    // $t3 := borrow_field<Option::Option<#0>>.vec($t2)
    call $t3 := $BorrowField($t2, $Option_Option_vec);

    // $t4 := 0
    $t4 := $Integer(0);

    // $t5 := read_ref($t3)
    call $t5 := $ReadRef($t3);

    // ($t6, $t5) := Vector::borrow_mut<#0>($t5, $t4)
    call $t6, $t5 := $Vector_borrow_mut($tv0, $t5, $t4);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,5549):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t3, $t5)
    call $t3 := $WriteRef($t3, $t5);

    // splice[0 -> $t3]($t6)
    call $t6 := $Splice1(0, $t3, $t6);

    // write_back[Reference($t2)]($t3)
    call $t2 := $WritebackToReference($t3, $t2);

    // write_back[LocalRoot($t1)]($t2)
    call $t1 := $WritebackToValue($t2, 1, $t1);

    // return ($t6, $t1)
    $ret0 := $t6;
    if (true) {
     $trace_temp := $Dereference($ret0);
      assume {:print "$track_local(6,5541,7):", $trace_temp} true;
    }
    $ret1 := $t1;
    if (true) {
     $trace_temp := $ret1;
      assume {:print "$track_local(6,5541,8):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultMutation;
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $Option_borrow_mut_$direct_inter($tv0: $TypeValue, t: $Value) returns ($ret0: $Mutation, $ret1: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Dereference($ret0), $Option_spec_get($tv0, $ret1)))));
ensures $Option_Option_$is_well_formed($ret1);

procedure {:inline 1} $Option_borrow_mut_$direct_intra($tv0: $TypeValue, t: $Value) returns ($ret0: $Mutation, $ret1: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Dereference($ret0), $Option_spec_get($tv0, $ret1)))));
ensures $Option_Option_$is_well_formed($ret1);

procedure {:inline 1} $Option_borrow_mut($tv0: $TypeValue, t: $Value) returns ($ret0: $Mutation, $ret1: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Dereference($ret0), $Option_spec_get($tv0, $ret1)))));
ensures $Option_Option_$is_well_formed($ret1);

procedure {:inline 1} $Option_contains_$def($tv0: $TypeValue, t: $Value, e_ref: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t2: $Value; // $Option_Option_type_value($tv0)
    var $t3: $Value; // $tv0
    var $t4: $Value; // $Vector_type_value($tv0)
    var $t5: $Value; // $BooleanType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,2435,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := e_ref;
      assume {:print "$track_local(6,2435,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(t)
    call $t2 := $CopyOrMoveValue(t);

    // $t3 := move(e_ref)
    call $t3 := $CopyOrMoveValue(e_ref);

    // $t4 := get_field<Option::Option<#0>>.vec($t2)
    call $t4 := $GetFieldFromValue($t2, $Option_Option_vec);

    // $t5 := Vector::contains<#0>($t4, $t3)
    call $t5 := $Vector_contains($tv0, $t4, $t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,2526):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t5
    $ret0 := $t5;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,2518,6):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Option_contains_$direct_inter($tv0: $TypeValue, t: $Value, e_ref: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Boolean(b#$Boolean($Option_spec_is_some($tv0, t)) && b#$Boolean($Boolean($IsEqual($Option_spec_get($tv0, t), e_ref))))))));
ensures is#$Boolean($ret0);

procedure {:inline 1} $Option_contains_$direct_intra($tv0: $TypeValue, t: $Value, e_ref: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Boolean(b#$Boolean($Option_spec_is_some($tv0, t)) && b#$Boolean($Boolean($IsEqual($Option_spec_get($tv0, t), e_ref))))))));
ensures is#$Boolean($ret0);

procedure {:inline 1} $Option_contains($tv0: $TypeValue, t: $Value, e_ref: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Boolean(b#$Boolean($Option_spec_is_some($tv0, t)) && b#$Boolean($Boolean($IsEqual($Option_spec_get($tv0, t), e_ref))))))));
ensures is#$Boolean($ret0);

procedure {:inline 1} $Option_swap_$def($tv0: $TypeValue, t: $Value, e: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    // declare local variables
    var old_value: $Value; // $tv0
    var vec_ref: $Mutation; // ReferenceType($Vector_type_value($tv0))
    var $t4: $Value; // $Option_Option_type_value($tv0)
    var $t5: $Value; // $tv0
    var $t6: $Mutation; // ReferenceType($Option_Option_type_value($tv0))
    var $t7: $Value; // $Vector_type_value($tv0)
    var $t8: $Value; // $tv0
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,5838,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := e;
      assume {:print "$track_local(6,5838,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t4 := move(t)
    call $t4 := $CopyOrMoveValue(t);

    // $t5 := move(e)
    call $t5 := $CopyOrMoveValue(e);

    // $t6 := borrow_local($t4)
    call $t6 := $BorrowLoc(4, $t4);

    // unpack_ref($t6)
    call $Option_Option_$unpack_ref($tv0, $Dereference($t6));

    // vec_ref := borrow_field<Option::Option<#0>>.vec($t6)
    call vec_ref := $BorrowField($t6, $Option_Option_vec);

    // unpack_ref(vec_ref)

    // $t7 := read_ref(vec_ref)
    call $t7 := $ReadRef(vec_ref);

    // ($t8, $t7) := Vector::pop_back<#0>($t7)
    call $t8, $t7 := $Vector_pop_back($tv0, $t7);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,5977):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref(vec_ref, $t7)
    call vec_ref := $WriteRef(vec_ref, $t7);
    if (true) {
     $trace_temp := $Dereference(vec_ref);
      assume {:print "$track_local(6,5977,3):", $trace_temp} true;
    }

    // old_value := $t8
    call old_value := $CopyOrMoveValue($t8);
    if (true) {
     $trace_temp := old_value;
      assume {:print "$track_local(6,5957,2):", $trace_temp} true;
    }

    // $t7 := read_ref(vec_ref)
    call $t7 := $ReadRef(vec_ref);

    // $t7 := Vector::push_back<#0>($t7, $t5)
    call $t7 := $Vector_push_back($tv0, $t7, $t5);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,6012):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref(vec_ref, $t7)
    call vec_ref := $WriteRef(vec_ref, $t7);
    if (true) {
     $trace_temp := $Dereference(vec_ref);
      assume {:print "$track_local(6,6012,3):", $trace_temp} true;
    }

    // pack_ref(vec_ref)

    // write_back[Reference($t6)](vec_ref)
    call $t6 := $WritebackToReference(vec_ref, $t6);

    // pack_ref($t6)
    call $Option_Option_$pack_ref($tv0, $Dereference($t6));

    // write_back[LocalRoot($t4)]($t6)
    call $t4 := $WritebackToValue($t6, 4, $t4);

    // return (old_value, $t4)
    $ret0 := old_value;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,6043,9):", $trace_temp} true;
    }
    $ret1 := $t4;
    if (true) {
     $trace_temp := $ret1;
      assume {:print "$track_local(6,6043,10):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $Option_swap_$direct_inter($tv0: $TypeValue, t: $Value, e: $Value) returns ($ret0: $Value, $ret1: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_some($tv0, $ret1)));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Option_spec_get($tv0, $ret1), e))));
ensures $Option_Option_$is_well_formed($ret1);

procedure {:inline 1} $Option_swap_$direct_intra($tv0: $TypeValue, t: $Value, e: $Value) returns ($ret0: $Value, $ret1: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_some($tv0, $ret1)));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Option_spec_get($tv0, $ret1), e))));
ensures $Option_Option_$is_well_formed($ret1);

procedure {:inline 1} $Option_swap($tv0: $TypeValue, t: $Value, e: $Value) returns ($ret0: $Value, $ret1: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_some($tv0, $ret1)));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Option_spec_get($tv0, $ret1), e))));
ensures $Option_Option_$is_well_formed($ret1);

procedure {:inline 1} $Option_borrow_with_default_$def($tv0: $TypeValue, t: $Value, default_ref: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var tmp#$2: $Value; // $tv0
    var vec_ref: $Value; // $Vector_type_value($tv0)
    var $t4: $Value; // $Option_Option_type_value($tv0)
    var $t5: $Value; // $tv0
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,3462,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := default_ref;
      assume {:print "$track_local(6,3462,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t4 := move(t)
    call $t4 := $CopyOrMoveValue(t);

    // $t5 := move(default_ref)
    call $t5 := $CopyOrMoveValue(default_ref);

    // vec_ref := get_field<Option::Option<#0>>.vec($t4)
    call vec_ref := $GetFieldFromValue($t4, $Option_Option_vec);
    if (true) {
     $trace_temp := vec_ref;
      assume {:print "$track_local(6,3580,3):", $trace_temp} true;
    }

    // $t6 := Vector::is_empty<#0>(vec_ref)
    call $t6 := $Vector_is_empty($tv0, vec_ref);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,3608):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t6) goto L0 else goto L1
    if (b#$Boolean($t6)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // destroy(vec_ref)

    // tmp#$2 := $t5
    call tmp#$2 := $CopyOrMoveValue($t5);
    if (true) {
     $trace_temp := tmp#$2;
      assume {:print "$track_local(6,3596,2):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L2:
L2:

    // destroy($t5)

    // $t7 := 0
    $t7 := $Integer(0);

    // tmp#$2 := Vector::borrow<#0>(vec_ref, $t7)
    call tmp#$2 := $Vector_borrow($tv0, vec_ref, $t7);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,3660):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // goto L3
    goto L3;

    // L3:
L3:

    // return tmp#$2
    $ret0 := tmp#$2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,3596,8):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Option_borrow_with_default_$direct_inter($tv0: $TypeValue, t: $Value, default_ref: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_none($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, default_ref))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_some($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, t)))))));

procedure {:inline 1} $Option_borrow_with_default_$direct_intra($tv0: $TypeValue, t: $Value, default_ref: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_none($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, default_ref))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_some($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, t)))))));

procedure {:inline 1} $Option_borrow_with_default($tv0: $TypeValue, t: $Value, default_ref: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_none($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, default_ref))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_some($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, t)))))));

procedure {:inline 1} $Option_destroy_none_$def($tv0: $TypeValue, t: $Value) returns ()
{
    // declare local variables
    var vec: $Value; // $Vector_type_value($tv0)
    var $t2: $Value; // $Option_Option_type_value($tv0)
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,7295,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(t)
    call $t2 := $CopyOrMoveValue(t);

    // vec := unpack Option::Option<#0>($t2)
    call vec := $Option_Option_unpack($tv0, $t2);
    if (true) {
     $trace_temp := vec;
      assume {:print "$track_local(6,7362,1):", $trace_temp} true;
    }

    // Vector::destroy_empty<#0>(vec)
    call $Vector_destroy_empty($tv0, vec);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,7398):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Option_destroy_none_$direct_inter($tv0: $TypeValue, t: $Value) returns ()
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_some($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_some($tv0, t))));

procedure {:inline 1} $Option_destroy_none_$direct_intra($tv0: $TypeValue, t: $Value) returns ()
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_some($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_some($tv0, t))));

procedure {:inline 1} $Option_destroy_none($tv0: $TypeValue, t: $Value) returns ()
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_some($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_some($tv0, t))));

procedure {:inline 1} $Option_destroy_some_$def($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var elem: $Value; // $tv0
    var vec: $Value; // $Vector_type_value($tv0)
    var $t3: $Value; // $Option_Option_type_value($tv0)
    var $t4: $Mutation; // ReferenceType($Vector_type_value($tv0))
    var $t5: $Value; // $Vector_type_value($tv0)
    var $t6: $Value; // $tv0
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,6893,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(t)
    call $t3 := $CopyOrMoveValue(t);

    // vec := unpack Option::Option<#0>($t3)
    call vec := $Option_Option_unpack($tv0, $t3);
    if (true) {
     $trace_temp := vec;
      assume {:print "$track_local(6,6969,2):", $trace_temp} true;
    }

    // $t4 := borrow_local(vec)
    call $t4 := $BorrowLoc(2, vec);

    // unpack_ref($t4)

    // $t5 := read_ref($t4)
    call $t5 := $ReadRef($t4);

    // ($t6, $t5) := Vector::pop_back<#0>($t5)
    call $t6, $t5 := $Vector_pop_back($tv0, $t5);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,7016):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t4, $t5)
    call $t4 := $WriteRef($t4, $t5);

    // pack_ref($t4)

    // write_back[LocalRoot(vec)]($t4)
    call vec := $WritebackToValue($t4, 2, vec);

    // elem := $t6
    call elem := $CopyOrMoveValue($t6);
    if (true) {
     $trace_temp := elem;
      assume {:print "$track_local(6,7001,1):", $trace_temp} true;
    }

    // Vector::destroy_empty<#0>(vec)
    call $Vector_destroy_empty($tv0, vec);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,7052):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return elem
    $ret0 := elem;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,7080,7):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Option_destroy_some_$direct_inter($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))));

procedure {:inline 1} $Option_destroy_some_$direct_intra($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))));

procedure {:inline 1} $Option_destroy_some($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))));

procedure {:inline 1} $Option_destroy_with_default_$def($tv0: $TypeValue, t: $Value, default: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var tmp#$2: $Value; // $tv0
    var vec: $Value; // $Vector_type_value($tv0)
    var $t4: $Value; // $Option_Option_type_value($tv0)
    var $t5: $Value; // $tv0
    var $t6: $Mutation; // ReferenceType($Vector_type_value($tv0))
    var $t7: $Value; // $Vector_type_value($tv0)
    var $t8: $Value; // $BooleanType()
    var $t9: $Mutation; // ReferenceType($Vector_type_value($tv0))
    var $t10: $Value; // $Vector_type_value($tv0)
    var $t11: $Value; // $tv0
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,6349,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := default;
      assume {:print "$track_local(6,6349,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t4 := move(t)
    call $t4 := $CopyOrMoveValue(t);

    // $t5 := move(default)
    call $t5 := $CopyOrMoveValue(default);

    // vec := unpack Option::Option<#0>($t4)
    call vec := $Option_Option_unpack($tv0, $t4);
    if (true) {
     $trace_temp := vec;
      assume {:print "$track_local(6,6461,3):", $trace_temp} true;
    }

    // $t6 := borrow_local(vec)
    call $t6 := $BorrowLoc(3, vec);

    // unpack_ref($t6)

    // $t7 := read_ref($t6)
    call $t7 := $ReadRef($t6);

    // pack_ref($t6)

    // $t8 := Vector::is_empty<#0>($t7)
    call $t8 := $Vector_is_empty($tv0, $t7);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,6501):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t8) goto L0 else goto L1
    if (b#$Boolean($t8)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // tmp#$2 := $t5
    call tmp#$2 := $CopyOrMoveValue($t5);
    if (true) {
     $trace_temp := tmp#$2;
      assume {:print "$track_local(6,6489,2):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t9 := borrow_local(vec)
    call $t9 := $BorrowLoc(3, vec);

    // unpack_ref($t9)

    // $t10 := read_ref($t9)
    call $t10 := $ReadRef($t9);

    // ($t11, $t10) := Vector::pop_back<#0>($t10)
    call $t11, $t10 := $Vector_pop_back($tv0, $t10);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,6550):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t9, $t10)
    call $t9 := $WriteRef($t9, $t10);

    // pack_ref($t9)

    // write_back[LocalRoot(vec)]($t9)
    call vec := $WritebackToValue($t9, 3, vec);

    // tmp#$2 := $t11
    call tmp#$2 := $CopyOrMoveValue($t11);
    if (true) {
     $trace_temp := tmp#$2;
      assume {:print "$track_local(6,6489,2):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L3:
L3:

    // return tmp#$2
    $ret0 := tmp#$2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,6489,12):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Option_destroy_with_default_$direct_inter($tv0: $TypeValue, t: $Value, default: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_none($tv0, old(t))) ==> b#$Boolean($Boolean($IsEqual($ret0, default))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_some($tv0, old(t))) ==> b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))))));

procedure {:inline 1} $Option_destroy_with_default_$direct_intra($tv0: $TypeValue, t: $Value, default: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_none($tv0, old(t))) ==> b#$Boolean($Boolean($IsEqual($ret0, default))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_some($tv0, old(t))) ==> b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))))));

procedure {:inline 1} $Option_destroy_with_default($tv0: $TypeValue, t: $Value, default: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_none($tv0, old(t))) ==> b#$Boolean($Boolean($IsEqual($ret0, default))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_some($tv0, old(t))) ==> b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))))));

procedure {:inline 1} $Option_extract_$def($tv0: $TypeValue, t: $Value) returns ($ret0: $Value, $ret1: $Value)
{
    // declare local variables
    var $t1: $Value; // $Option_Option_type_value($tv0)
    var $t2: $Mutation; // ReferenceType($Option_Option_type_value($tv0))
    var $t3: $Mutation; // ReferenceType($Vector_type_value($tv0))
    var $t4: $Value; // $Vector_type_value($tv0)
    var $t5: $Value; // $tv0
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,5075,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(t)
    call $t1 := $CopyOrMoveValue(t);

    // $t2 := borrow_local($t1)
    call $t2 := $BorrowLoc(1, $t1);

    // unpack_ref($t2)
    call $Option_Option_$unpack_ref($tv0, $Dereference($t2));

    // $t3 := borrow_field<Option::Option<#0>>.vec($t2)
    call $t3 := $BorrowField($t2, $Option_Option_vec);

    // unpack_ref($t3)

    // $t4 := read_ref($t3)
    call $t4 := $ReadRef($t3);

    // ($t5, $t4) := Vector::pop_back<#0>($t4)
    call $t5, $t4 := $Vector_pop_back($tv0, $t4);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,5155):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t3, $t4)
    call $t3 := $WriteRef($t3, $t4);

    // pack_ref($t3)

    // write_back[Reference($t2)]($t3)
    call $t2 := $WritebackToReference($t3, $t2);

    // pack_ref($t2)
    call $Option_Option_$pack_ref($tv0, $Dereference($t2));

    // write_back[LocalRoot($t1)]($t2)
    call $t1 := $WritebackToValue($t2, 1, $t1);

    // return ($t5, $t1)
    $ret0 := $t5;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,5147,6):", $trace_temp} true;
    }
    $ret1 := $t1;
    if (true) {
     $trace_temp := $ret1;
      assume {:print "$track_local(6,5147,7):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
    $ret1 := $DefaultValue();
}

procedure {:inline 1} $Option_extract_$direct_inter($tv0: $TypeValue, t: $Value) returns ($ret0: $Value, $ret1: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_none($tv0, $ret1)));
ensures $Option_Option_$is_well_formed($ret1);

procedure {:inline 1} $Option_extract_$direct_intra($tv0: $TypeValue, t: $Value) returns ($ret0: $Value, $ret1: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_none($tv0, $ret1)));
ensures $Option_Option_$is_well_formed($ret1);

procedure {:inline 1} $Option_extract($tv0: $TypeValue, t: $Value) returns ($ret0: $Value, $ret1: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_none($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_none($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, old(t))))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_none($tv0, $ret1)));
ensures $Option_Option_$is_well_formed($ret1);

procedure {:inline 1} $Option_fill_$def($tv0: $TypeValue, t: $Value, e: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var vec_ref: $Mutation; // ReferenceType($Vector_type_value($tv0))
    var $t3: $Value; // $Option_Option_type_value($tv0)
    var $t4: $Value; // $tv0
    var $t5: $Mutation; // ReferenceType($Option_Option_type_value($tv0))
    var $t6: $Value; // $Vector_type_value($tv0)
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $Vector_type_value($tv0)
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,4555,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := e;
      assume {:print "$track_local(6,4555,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(t)
    call $t3 := $CopyOrMoveValue(t);

    // $t4 := move(e)
    call $t4 := $CopyOrMoveValue(e);

    // $t5 := borrow_local($t3)
    call $t5 := $BorrowLoc(3, $t3);

    // unpack_ref($t5)
    call $Option_Option_$unpack_ref($tv0, $Dereference($t5));

    // vec_ref := borrow_field<Option::Option<#0>>.vec($t5)
    call vec_ref := $BorrowField($t5, $Option_Option_vec);

    // unpack_ref(vec_ref)

    // $t6 := read_ref(vec_ref)
    call $t6 := $ReadRef(vec_ref);

    // $t7 := Vector::is_empty<#0>($t6)
    call $t7 := $Vector_is_empty($tv0, $t6);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,4673):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t7) goto L0 else goto L1
    if (b#$Boolean($t7)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // destroy(vec_ref)

    // pack_ref(vec_ref)

    // pack_ref($t5)
    call $Option_Option_$pack_ref($tv0, $Dereference($t5));

    // $t8 := 101
    $t8 := $Integer(101);

    // abort($t8)
    $abort_code := i#$Integer($t8);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,4735):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // $t9 := read_ref(vec_ref)
    call $t9 := $ReadRef(vec_ref);

    // $t9 := Vector::push_back<#0>($t9, $t4)
    call $t9 := $Vector_push_back($tv0, $t9, $t4);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,4700):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref(vec_ref, $t9)
    call vec_ref := $WriteRef(vec_ref, $t9);
    if (true) {
     $trace_temp := $Dereference(vec_ref);
      assume {:print "$track_local(6,4700,2):", $trace_temp} true;
    }

    // pack_ref(vec_ref)

    // write_back[Reference($t5)](vec_ref)
    call $t5 := $WritebackToReference(vec_ref, $t5);

    // pack_ref($t5)
    call $Option_Option_$pack_ref($tv0, $Dereference($t5));

    // write_back[LocalRoot($t3)]($t5)
    call $t3 := $WritebackToValue($t5, 3, $t3);

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,4661,10):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Option_fill_$direct_inter($tv0: $TypeValue, t: $Value, e: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_some($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_some($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_some($tv0, $ret0)));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Option_spec_get($tv0, $ret0), e))));
ensures $Option_Option_$is_well_formed($ret0);

procedure {:inline 1} $Option_fill_$direct_intra($tv0: $TypeValue, t: $Value, e: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_some($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_some($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_some($tv0, $ret0)));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Option_spec_get($tv0, $ret0), e))));
ensures $Option_Option_$is_well_formed($ret0);

procedure {:inline 1} $Option_fill($tv0: $TypeValue, t: $Value, e: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Option_spec_is_some($tv0, t))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Option_spec_is_some($tv0, t))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_some($tv0, $ret0)));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($Option_spec_get($tv0, $ret0), e))));
ensures $Option_Option_$is_well_formed($ret0);

procedure {:inline 1} $Option_get_with_default_$def($tv0: $TypeValue, t: $Value, default: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var tmp#$2: $Value; // $tv0
    var vec_ref: $Value; // $Vector_type_value($tv0)
    var $t4: $Value; // $Option_Option_type_value($tv0)
    var $t5: $Value; // $tv0
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $tv0
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,4010,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := default;
      assume {:print "$track_local(6,4010,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t4 := move(t)
    call $t4 := $CopyOrMoveValue(t);

    // $t5 := move(default)
    call $t5 := $CopyOrMoveValue(default);

    // vec_ref := get_field<Option::Option<#0>>.vec($t4)
    call vec_ref := $GetFieldFromValue($t4, $Option_Option_vec);
    if (true) {
     $trace_temp := vec_ref;
      assume {:print "$track_local(6,4129,3):", $trace_temp} true;
    }

    // $t6 := Vector::is_empty<#0>(vec_ref)
    call $t6 := $Vector_is_empty($tv0, vec_ref);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,4157):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t6) goto L0 else goto L1
    if (b#$Boolean($t6)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // destroy(vec_ref)

    // tmp#$2 := $t5
    call tmp#$2 := $CopyOrMoveValue($t5);
    if (true) {
     $trace_temp := tmp#$2;
      assume {:print "$track_local(6,4145,2):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t7 := 0
    $t7 := $Integer(0);

    // $t8 := Vector::borrow<#0>(vec_ref, $t7)
    call $t8 := $Vector_borrow($tv0, vec_ref, $t7);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,4206):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // tmp#$2 := $t8
    call tmp#$2 := $CopyOrMoveValue($t8);
    if (true) {
     $trace_temp := tmp#$2;
      assume {:print "$track_local(6,4145,2):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L3:
L3:

    // return tmp#$2
    $ret0 := tmp#$2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,4145,9):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Option_get_with_default_$direct_inter($tv0: $TypeValue, t: $Value, default: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_none($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, default))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_some($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, t)))))));

procedure {:inline 1} $Option_get_with_default_$direct_intra($tv0: $TypeValue, t: $Value, default: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_none($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, default))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_some($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, t)))))));

procedure {:inline 1} $Option_get_with_default($tv0: $TypeValue, t: $Value, default: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_none($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, default))))));
ensures !$abort_flag ==> (b#$Boolean($Boolean(b#$Boolean($Option_spec_is_some($tv0, t)) ==> b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_get($tv0, t)))))));

procedure {:inline 1} $Option_is_none_$def($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $Option_Option_type_value($tv0)
    var $t2: $Value; // $Vector_type_value($tv0)
    var $t3: $Value; // $BooleanType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,1562,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(t)
    call $t1 := $CopyOrMoveValue(t);

    // $t2 := get_field<Option::Option<#0>>.vec($t1)
    call $t2 := $GetFieldFromValue($t1, $Option_Option_vec);

    // $t3 := Vector::is_empty<#0>($t2)
    call $t3 := $Vector_is_empty($tv0, $t2);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,1635):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,1627,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Option_is_none_$direct_inter($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_is_none($tv0, t)))));
ensures is#$Boolean($ret0);

procedure {:inline 1} $Option_is_none_$direct_intra($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_is_none($tv0, t)))));
ensures is#$Boolean($ret0);

procedure {:inline 1} $Option_is_none($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_is_none($tv0, t)))));
ensures is#$Boolean($ret0);

procedure {:inline 1} $Option_is_some_$def($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $Option_Option_type_value($tv0)
    var $t2: $Value; // $Vector_type_value($tv0)
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $BooleanType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := t;
      assume {:print "$track_local(6,1958,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(t)
    call $t1 := $CopyOrMoveValue(t);

    // $t2 := get_field<Option::Option<#0>>.vec($t1)
    call $t2 := $GetFieldFromValue($t1, $Option_Option_vec);

    // $t3 := Vector::is_empty<#0>($t2)
    call $t3 := $Vector_is_empty($tv0, $t2);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,2032):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t4 := !($t3)
    call $t4 := $Not($t3);

    // return $t4
    $ret0 := $t4;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,2023,5):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Option_is_some_$direct_inter($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_is_some($tv0, t)))));
ensures is#$Boolean($ret0);

procedure {:inline 1} $Option_is_some_$direct_intra($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_is_some($tv0, t)))));
ensures is#$Boolean($ret0);

procedure {:inline 1} $Option_is_some($tv0: $TypeValue, t: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_is_some($tv0, t)))));
ensures is#$Boolean($ret0);

procedure {:inline 1} $Option_none_$def($tv0: $TypeValue) returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $Vector_type_value($tv0)
    var $t1: $Value; // $Option_Option_type_value($tv0)
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := Vector::empty<#0>()
    call $t0 := $Vector_empty($tv0);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(6,803):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t1 := pack Option::Option<#0>($t0)
    call $t1 := $Option_Option_pack(0, 0, 0, $tv0, $t0);

    // return $t1
    $ret0 := $t1;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,781,2):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Option_none_$direct_inter($tv0: $TypeValue) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_none($tv0)))));
ensures $Option_Option_$is_well_formed($ret0);

procedure {:inline 1} $Option_none_$direct_intra($tv0: $TypeValue) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_none($tv0)))));
ensures $Option_Option_$is_well_formed($ret0);

procedure {:inline 1} $Option_none($tv0: $TypeValue) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Option_spec_none($tv0)))));
ensures $Option_Option_$is_well_formed($ret0);

procedure {:inline 1} $Option_some_$def($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $tv0
    var $t2: $Value; // $Vector_type_value($tv0)
    var $t3: $Value; // $Option_Option_type_value($tv0)
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := e;
      assume {:print "$track_local(6,1126,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(e)
    call $t1 := $CopyOrMoveValue(e);

    // $t2 := Vector::singleton<#0>($t1)
    call $t2 := $Vector_singleton($tv0, $t1);
    if ($abort_flag) {
      goto Abort;
    }

    // $t3 := pack Option::Option<#0>($t2)
    call $t3 := $Option_Option_pack(0, 0, 0, $tv0, $t2);

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(6,1190,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Option_some_$direct_inter($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
{
    call $ret0 := $Option_some_$def($tv0, e);
}


procedure {:inline 1} $Option_some_$direct_intra($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
{
    call $ret0 := $Option_some_$def($tv0, e);
}


procedure {:inline 1} $Option_some($tv0: $TypeValue, e: $Value) returns ($ret0: $Value)
{
    call $ret0 := $Option_some_$def($tv0, e);
}




// ** spec vars of module SCS



// ** spec funs of module SCS



// ** structs of module SCS



// ** functions of module SCS



// ** spec vars of module Event



// ** spec funs of module Event



// ** structs of module Event

const unique $Event_EventHandle: $TypeName;
const $Event_EventHandle_counter: $FieldName;
axiom $Event_EventHandle_counter == 0;
const $Event_EventHandle_guid: $FieldName;
axiom $Event_EventHandle_guid == 1;
function $Event_EventHandle_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Event_EventHandle, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1))
}
var $Event_EventHandle_$memory: $Memory;
var $Event_EventHandle_$memory_$old: $Memory;
function {:inline} $Event_EventHandle_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 2
      && $IsValidU64($SelectField($this, $Event_EventHandle_counter))
      && $Vector_$is_well_formed($SelectField($this, $Event_EventHandle_guid)) && (forall $$0: int :: {$select_vector($SelectField($this, $Event_EventHandle_guid),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $Event_EventHandle_guid)) ==> $IsValidU8($select_vector($SelectField($this, $Event_EventHandle_guid),$$0)))
}
function {:inline} $Event_EventHandle_$invariant_holds($this: $Value): bool {
    true
}

function {:inline} $Event_EventHandle_$is_well_formed($this: $Value): bool {
    $Event_EventHandle_$is_well_typed($this) && $Event_EventHandle_$invariant_holds($this)}

procedure {:inline 1} $Event_EventHandle_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, counter: $Value, guid: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(counter);
    assume $Vector_$is_well_formed(guid) && (forall $$0: int :: {$select_vector(guid,$$0)} $$0 >= 0 && $$0 < $vlen(guid) ==> $IsValidU8($select_vector(guid,$$0)));
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := counter][1 := guid], 2));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $Event_EventHandle_unpack($tv0: $TypeValue, $struct: $Value) returns (counter: $Value, guid: $Value)
{
    assume is#$Vector($struct);
    counter := $SelectField($struct, $Event_EventHandle_counter);
    assume $IsValidU64(counter);
    guid := $SelectField($struct, $Event_EventHandle_guid);
    assume $Vector_$is_well_formed(guid) && (forall $$0: int :: {$select_vector(guid,$$0)} $$0 >= 0 && $$0 < $vlen(guid) ==> $IsValidU8($select_vector(guid,$$0)));
}

const unique $Event_EventHandleGenerator: $TypeName;
const $Event_EventHandleGenerator_counter: $FieldName;
axiom $Event_EventHandleGenerator_counter == 0;
const $Event_EventHandleGenerator_addr: $FieldName;
axiom $Event_EventHandleGenerator_addr == 1;
function $Event_EventHandleGenerator_type_value(): $TypeValue {
    $StructType($Event_EventHandleGenerator, $EmptyTypeValueArray)
}
var $Event_EventHandleGenerator_$memory: $Memory;
var $Event_EventHandleGenerator_$memory_$old: $Memory;
function {:inline} $Event_EventHandleGenerator_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 2
      && $IsValidU64($SelectField($this, $Event_EventHandleGenerator_counter))
      && is#$Address($SelectField($this, $Event_EventHandleGenerator_addr))
}
function {:inline} $Event_EventHandleGenerator_$invariant_holds($this: $Value): bool {
    true
}

function {:inline} $Event_EventHandleGenerator_$is_well_formed($this: $Value): bool {
    $Event_EventHandleGenerator_$is_well_typed($this) && $Event_EventHandleGenerator_$invariant_holds($this)}

procedure {:inline 1} $Event_EventHandleGenerator_pack($file_id: int, $byte_index: int, $var_idx: int, counter: $Value, addr: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(counter);
    assume is#$Address(addr);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := counter][1 := addr], 2));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $Event_EventHandleGenerator_unpack($struct: $Value) returns (counter: $Value, addr: $Value)
{
    assume is#$Vector($struct);
    counter := $SelectField($struct, $Event_EventHandleGenerator_counter);
    assume $IsValidU64(counter);
    addr := $SelectField($struct, $Event_EventHandleGenerator_addr);
    assume is#$Address(addr);
}



// ** functions of module Event



// ** spec vars of module Errors



// ** spec funs of module Errors



// ** structs of module Errors



// ** functions of module Errors

procedure {:inline 1} $Errors_already_published_$def(reason: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := reason;
      assume {:print "$track_local(4,4449,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(reason)
    call $t1 := $CopyOrMoveValue(reason);

    // $t2 := 6
    $t2 := $Integer(6);

    // $t3 := Errors::make($t2, $t1)
    call $t3 := $Errors_make($t2, $t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1525):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(4,4498,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Errors_already_published_$direct_inter(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(6)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_already_published_$direct_intra(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(6)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_already_published(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(6)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_custom_$def(reason: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := reason;
      assume {:print "$track_local(4,5305,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(reason)
    call $t1 := $CopyOrMoveValue(reason);

    // $t2 := 255
    $t2 := $Integer(255);

    // $t3 := Errors::make($t2, $t1)
    call $t3 := $Errors_make($t2, $t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1525):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(4,5343,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Errors_custom_$direct_inter(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(255)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_custom_$direct_intra(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(255)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_custom(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(255)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_internal_$def(reason: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := reason;
      assume {:print "$track_local(4,5114,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(reason)
    call $t1 := $CopyOrMoveValue(reason);

    // $t2 := 10
    $t2 := $Integer(10);

    // $t3 := Errors::make($t2, $t1)
    call $t3 := $Errors_make($t2, $t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1525):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(4,5154,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Errors_internal_$direct_inter(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(10)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_internal_$direct_intra(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(10)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_internal(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(10)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_invalid_argument_$def(reason: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := reason;
      assume {:print "$track_local(4,4676,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(reason)
    call $t1 := $CopyOrMoveValue(reason);

    // $t2 := 7
    $t2 := $Integer(7);

    // $t3 := Errors::make($t2, $t1)
    call $t3 := $Errors_make($t2, $t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1525):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(4,4724,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Errors_invalid_argument_$direct_inter(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(7)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_invalid_argument_$direct_intra(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(7)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_invalid_argument(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(7)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_invalid_state_$def(reason: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := reason;
      assume {:print "$track_local(4,3358,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(reason)
    call $t1 := $CopyOrMoveValue(reason);

    // $t2 := 1
    $t2 := $Integer(1);

    // $t3 := Errors::make($t2, $t1)
    call $t3 := $Errors_make($t2, $t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1525):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(4,3403,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Errors_invalid_state_$direct_inter(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(1)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_invalid_state_$direct_intra(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(1)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_invalid_state(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(1)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_limit_exceeded_$def(reason: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := reason;
      assume {:print "$track_local(4,4899,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(reason)
    call $t1 := $CopyOrMoveValue(reason);

    // $t2 := 8
    $t2 := $Integer(8);

    // $t3 := Errors::make($t2, $t1)
    call $t3 := $Errors_make($t2, $t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1525):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(4,4945,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Errors_limit_exceeded_$direct_inter(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(8)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_limit_exceeded_$direct_intra(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(8)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_limit_exceeded(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(8)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_make_$def(category: $Value, reason: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $IntegerType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := category;
      assume {:print "$track_local(4,1521,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := reason;
      assume {:print "$track_local(4,1521,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(category)
    call $t2 := $CopyOrMoveValue(category);

    // $t3 := move(reason)
    call $t3 := $CopyOrMoveValue(reason);

    // $t4 := (u64)($t2)
    call $t4 := $CastU64($t2);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1572):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t5 := 8
    $t5 := $Integer(8);

    // $t6 := <<($t3, $t5)
    call $t6 := $Shl($t3, $t5);

    // $t7 := +($t4, $t6)
    call $t7 := $AddU64($t4, $t6);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1590):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t7
    $ret0 := $t7;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(4,1572,8):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Errors_make_$direct_intra(category: $Value, reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, category))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_make(category: $Value, reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, category))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_not_published_$def(reason: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := reason;
      assume {:print "$track_local(4,4238,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(reason)
    call $t1 := $CopyOrMoveValue(reason);

    // $t2 := 5
    $t2 := $Integer(5);

    // $t3 := Errors::make($t2, $t1)
    call $t3 := $Errors_make($t2, $t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1525):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(4,4283,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Errors_not_published_$direct_inter(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(5)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_not_published_$direct_intra(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(5)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_not_published(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(5)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_requires_address_$def(reason: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := reason;
      assume {:print "$track_local(4,3569,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(reason)
    call $t1 := $CopyOrMoveValue(reason);

    // $t2 := 2
    $t2 := $Integer(2);

    // $t3 := Errors::make($t2, $t1)
    call $t3 := $Errors_make($t2, $t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1525):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(4,3617,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Errors_requires_address_$direct_inter(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(2)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_requires_address_$direct_intra(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(2)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_requires_address(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(2)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_requires_capability_$def(reason: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := reason;
      assume {:print "$track_local(4,4003,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(reason)
    call $t1 := $CopyOrMoveValue(reason);

    // $t2 := 4
    $t2 := $Integer(4);

    // $t3 := Errors::make($t2, $t1)
    call $t3 := $Errors_make($t2, $t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1525):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(4,4054,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Errors_requires_capability_$direct_inter(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(4)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_requires_capability_$direct_intra(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(4)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_requires_capability(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(4)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_requires_role_$def(reason: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $IntegerType()
    var $t3: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := reason;
      assume {:print "$track_local(4,3792,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(reason)
    call $t1 := $CopyOrMoveValue(reason);

    // $t2 := 3
    $t2 := $Integer(3);

    // $t3 := Errors::make($t2, $t1)
    call $t3 := $Errors_make($t2, $t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(4,1525):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(4,3837,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Errors_requires_role_$direct_inter(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(3)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_requires_role_$direct_intra(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(3)))));
ensures $IsValidU64($ret0);

procedure {:inline 1} $Errors_requires_role(reason: $Value) returns ($ret0: $Value)
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
ensures !$abort_flag ==> (b#$Boolean($Boolean($IsEqual($ret0, $Integer(3)))));
ensures $IsValidU64($ret0);



// ** spec vars of module Config



// ** spec funs of module Config

function {:inline} $Config_spec_get($Config_Config_$memory: $Memory, $tv0: $TypeValue, addr: $Value): $Value {
    $SelectField($ResourceValue($Config_Config_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), addr), $Config_Config_payload)
}

function {:inline} $Config_spec_cap($Config_ModifyConfigCapabilityHolder_$memory: $Memory, $tv0: $TypeValue, addr: $Value): $Value {
    $SelectField($ResourceValue($Config_ModifyConfigCapabilityHolder_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), addr), $Config_ModifyConfigCapabilityHolder_cap)
}



// ** structs of module Config

const unique $Config_Config: $TypeName;
const $Config_Config_payload: $FieldName;
axiom $Config_Config_payload == 0;
function $Config_Config_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Config_Config, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1))
}
var $Config_Config_$memory: $Memory;
var $Config_Config_$memory_$old: $Memory;
function {:inline} $Config_Config_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 1
}
function {:inline} $Config_Config_$invariant_holds($this: $Value): bool {
    true
}

function {:inline} $Config_Config_$is_well_formed($this: $Value): bool {
    $Config_Config_$is_well_typed($this) && $Config_Config_$invariant_holds($this)}

procedure {:inline 1} $Config_Config_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, payload: $Value) returns ($struct: $Value)
{
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := payload], 1));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $Config_Config_unpack($tv0: $TypeValue, $struct: $Value) returns (payload: $Value)
{
    assume is#$Vector($struct);
    payload := $SelectField($struct, $Config_Config_payload);
}

const unique $Config_ConfigChangeEvent: $TypeName;
const $Config_ConfigChangeEvent_account_address: $FieldName;
axiom $Config_ConfigChangeEvent_account_address == 0;
const $Config_ConfigChangeEvent_value: $FieldName;
axiom $Config_ConfigChangeEvent_value == 1;
function $Config_ConfigChangeEvent_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Config_ConfigChangeEvent, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1))
}
var $Config_ConfigChangeEvent_$memory: $Memory;
var $Config_ConfigChangeEvent_$memory_$old: $Memory;
function {:inline} $Config_ConfigChangeEvent_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 2
      && is#$Address($SelectField($this, $Config_ConfigChangeEvent_account_address))
}
function {:inline} $Config_ConfigChangeEvent_$invariant_holds($this: $Value): bool {
    true
}

function {:inline} $Config_ConfigChangeEvent_$is_well_formed($this: $Value): bool {
    $Config_ConfigChangeEvent_$is_well_typed($this) && $Config_ConfigChangeEvent_$invariant_holds($this)}

procedure {:inline 1} $Config_ConfigChangeEvent_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, account_address: $Value, value: $Value) returns ($struct: $Value)
{
    assume is#$Address(account_address);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := account_address][1 := value], 2));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $Config_ConfigChangeEvent_unpack($tv0: $TypeValue, $struct: $Value) returns (account_address: $Value, value: $Value)
{
    assume is#$Vector($struct);
    account_address := $SelectField($struct, $Config_ConfigChangeEvent_account_address);
    assume is#$Address(account_address);
    value := $SelectField($struct, $Config_ConfigChangeEvent_value);
}

const unique $Config_ModifyConfigCapability: $TypeName;
const $Config_ModifyConfigCapability_account_address: $FieldName;
axiom $Config_ModifyConfigCapability_account_address == 0;
const $Config_ModifyConfigCapability_events: $FieldName;
axiom $Config_ModifyConfigCapability_events == 1;
function $Config_ModifyConfigCapability_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Config_ModifyConfigCapability, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1))
}
var $Config_ModifyConfigCapability_$memory: $Memory;
var $Config_ModifyConfigCapability_$memory_$old: $Memory;
function {:inline} $Config_ModifyConfigCapability_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 2
      && is#$Address($SelectField($this, $Config_ModifyConfigCapability_account_address))
      && $Event_EventHandle_$is_well_typed($SelectField($this, $Config_ModifyConfigCapability_events))
}
function {:inline} $Config_ModifyConfigCapability_$invariant_holds($this: $Value): bool {
    $Event_EventHandle_$invariant_holds($SelectField($this, $Config_ModifyConfigCapability_events))
}

function {:inline} $Config_ModifyConfigCapability_$is_well_formed($this: $Value): bool {
    $Config_ModifyConfigCapability_$is_well_typed($this) && $Config_ModifyConfigCapability_$invariant_holds($this)}

procedure {:inline 1} $Config_ModifyConfigCapability_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, account_address: $Value, events: $Value) returns ($struct: $Value)
{
    assume is#$Address(account_address);
    assume $Event_EventHandle_$is_well_formed(events);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := account_address][1 := events], 2));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $Config_ModifyConfigCapability_unpack($tv0: $TypeValue, $struct: $Value) returns (account_address: $Value, events: $Value)
{
    assume is#$Vector($struct);
    account_address := $SelectField($struct, $Config_ModifyConfigCapability_account_address);
    assume is#$Address(account_address);
    events := $SelectField($struct, $Config_ModifyConfigCapability_events);
    assume $Event_EventHandle_$is_well_formed(events);
}

const unique $Config_ModifyConfigCapabilityHolder: $TypeName;
const $Config_ModifyConfigCapabilityHolder_cap: $FieldName;
axiom $Config_ModifyConfigCapabilityHolder_cap == 0;
function $Config_ModifyConfigCapabilityHolder_type_value($tv0: $TypeValue): $TypeValue {
    $StructType($Config_ModifyConfigCapabilityHolder, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1))
}
var $Config_ModifyConfigCapabilityHolder_$memory: $Memory;
var $Config_ModifyConfigCapabilityHolder_$memory_$old: $Memory;
function {:inline} $Config_ModifyConfigCapabilityHolder_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 1
      && $Option_Option_$is_well_typed($SelectField($this, $Config_ModifyConfigCapabilityHolder_cap))
}
function {:inline} $Config_ModifyConfigCapabilityHolder_$invariant_holds($this: $Value): bool {
    $Option_Option_$invariant_holds($SelectField($this, $Config_ModifyConfigCapabilityHolder_cap))
}

function {:inline} $Config_ModifyConfigCapabilityHolder_$is_well_formed($this: $Value): bool {
    $Config_ModifyConfigCapabilityHolder_$is_well_typed($this) && $Config_ModifyConfigCapabilityHolder_$invariant_holds($this)}

procedure {:inline 1} $Config_ModifyConfigCapabilityHolder_$unpack_ref_deep($tv0: $TypeValue, $before: $Value) {
    call $Option_Option_$unpack_ref($Config_ModifyConfigCapability_type_value($tv0), $SelectField($before, $Config_ModifyConfigCapabilityHolder_cap));
    assume $Config_ModifyConfigCapabilityHolder_$invariant_holds($before);
}

procedure {:inline 1} $Config_ModifyConfigCapabilityHolder_$unpack_ref($tv0: $TypeValue, $before: $Value) {
    assume $Config_ModifyConfigCapabilityHolder_$invariant_holds($before);
}

procedure {:inline 1} $Config_ModifyConfigCapabilityHolder_$pack_ref_deep($tv0: $TypeValue, $after: $Value) {
    call $Option_Option_$pack_ref($Config_ModifyConfigCapability_type_value($tv0), $SelectField($after, $Config_ModifyConfigCapabilityHolder_cap));
}

procedure {:inline 1} $Config_ModifyConfigCapabilityHolder_$pack_ref($tv0: $TypeValue, $after: $Value) {
}

procedure {:inline 1} $Config_ModifyConfigCapabilityHolder_pack($file_id: int, $byte_index: int, $var_idx: int, $tv0: $TypeValue, cap: $Value) returns ($struct: $Value)
{
    assume $Option_Option_$is_well_formed(cap);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := cap], 1));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $Config_ModifyConfigCapabilityHolder_unpack($tv0: $TypeValue, $struct: $Value) returns (cap: $Value)
{
    assume is#$Vector($struct);
    cap := $SelectField($struct, $Config_ModifyConfigCapabilityHolder_cap);
    assume $Option_Option_$is_well_formed(cap);
}



// ** functions of module Config

procedure {:inline 1} $Config_account_address_$def($tv0: $TypeValue, cap: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $Config_ModifyConfigCapability_type_value($tv0)
    var $t2: $Value; // $AddressType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(2,9310,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(cap)
    call $t1 := $CopyOrMoveValue(cap);

    // $t2 := get_field<Config::ModifyConfigCapability<#0>>.account_address($t1)
    call $t2 := $GetFieldFromValue($t1, $Config_ModifyConfigCapability_account_address);

    // return $t2
    $ret0 := $t2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(2,9422,3):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Config_account_address_$direct_inter($tv0: $TypeValue, cap: $Value) returns ($ret0: $Value)
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $ret0 := $Config_account_address_$def($tv0, cap);
}


procedure {:inline 1} $Config_account_address_$direct_intra($tv0: $TypeValue, cap: $Value) returns ($ret0: $Value)
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $ret0 := $Config_account_address_$def($tv0, cap);
}


procedure {:inline 1} $Config_account_address($tv0: $TypeValue, cap: $Value) returns ($ret0: $Value)
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $ret0 := $Config_account_address_$def($tv0, cap);
}


procedure {:inline 1} $Config_config_exist_by_address_$def($tv0: $TypeValue, addr: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $AddressType()
    var $t2: $Value; // $BooleanType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := addr;
      assume {:print "$track_local(2,1787,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(addr)
    call $t1 := $CopyOrMoveValue(addr);

    // $t2 := exists<Config::Config<#0>>($t1)
    $t2 := $ResourceExists($Config_Config_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $t1);

    // return $t2
    $ret0 := $t2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(2,1876,3):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Config_config_exist_by_address_$direct_inter($tv0: $TypeValue, addr: $Value) returns ($ret0: $Value)
{
    assume is#$Address(addr);

    call $ret0 := $Config_config_exist_by_address_$def($tv0, addr);
}


procedure {:inline 1} $Config_config_exist_by_address_$direct_intra($tv0: $TypeValue, addr: $Value) returns ($ret0: $Value)
{
    assume is#$Address(addr);

    call $ret0 := $Config_config_exist_by_address_$def($tv0, addr);
}


procedure {:inline 1} $Config_config_exist_by_address($tv0: $TypeValue, addr: $Value) returns ($ret0: $Value)
{
    assume is#$Address(addr);

    call $ret0 := $Config_config_exist_by_address_$def($tv0, addr);
}


procedure {:inline 1} $Config_destroy_modify_config_capability_$def($tv0: $TypeValue, cap: $Value) returns ()
{
    // declare local variables
    var events: $Value; // $Event_EventHandle_type_value($Config_ConfigChangeEvent_type_value($tv0))
    var $t2: $Value; // $Config_ModifyConfigCapability_type_value($tv0)
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $Event_EventHandle_type_value($Config_ConfigChangeEvent_type_value($tv0))
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(2,9001,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(cap)
    call $t2 := $CopyOrMoveValue(cap);

    // ($t3, $t4) := unpack Config::ModifyConfigCapability<#0>($t2)
    call $t3, $t4 := $Config_ModifyConfigCapability_unpack($tv0, $t2);



    // events := $t4
    call events := $CopyOrMoveValue($t4);
    if (true) {
     $trace_temp := events;
      assume {:print "$track_local(2,9166,1):", $trace_temp} true;
    }

    // destroy($t3)

    // Event::destroy_handle<Config::ConfigChangeEvent<#0>>(events)
    call $Event_destroy_handle($Config_ConfigChangeEvent_type_value($tv0), events);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,9196):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Config_destroy_modify_config_capability_$direct_inter($tv0: $TypeValue, cap: $Value) returns ()
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $Config_destroy_modify_config_capability_$def($tv0, cap);
}


procedure {:inline 1} $Config_destroy_modify_config_capability_$direct_intra($tv0: $TypeValue, cap: $Value) returns ()
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $Config_destroy_modify_config_capability_$def($tv0, cap);
}


procedure {:inline 1} $Config_destroy_modify_config_capability($tv0: $TypeValue, cap: $Value) returns ()
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $Config_destroy_modify_config_capability_$def($tv0, cap);
}


procedure {:inline 1} $Config_emit_config_change_event_$def($tv0: $TypeValue, cap: $Value, value: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t2: $Value; // $Config_ModifyConfigCapability_type_value($tv0)
    var $t3: $Value; // $tv0
    var $t4: $Mutation; // ReferenceType($Config_ModifyConfigCapability_type_value($tv0))
    var $t5: $Mutation; // ReferenceType($Event_EventHandle_type_value($Config_ConfigChangeEvent_type_value($tv0)))
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $Config_ConfigChangeEvent_type_value($tv0)
    var $t8: $Value; // $Event_EventHandle_type_value($Config_ConfigChangeEvent_type_value($tv0))
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(2,9550,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := value;
      assume {:print "$track_local(2,9550,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(cap)
    call $t2 := $CopyOrMoveValue(cap);

    // $t3 := move(value)
    call $t3 := $CopyOrMoveValue(value);

    // $t4 := borrow_local($t2)
    call $t4 := $BorrowLoc(2, $t2);

    // $t5 := borrow_field<Config::ModifyConfigCapability<#0>>.events($t4)
    call $t5 := $BorrowField($t4, $Config_ModifyConfigCapability_events);

    // $t6 := get_field<Config::ModifyConfigCapability<#0>>.account_address($t4)
    call $t6 := $GetFieldFromReference($t4, $Config_ModifyConfigCapability_account_address);

    // $t7 := pack Config::ConfigChangeEvent<#0>($t6, $t3)
    call $t7 := $Config_ConfigChangeEvent_pack(0, 0, 0, $tv0, $t6, $t3);

    // $t8 := read_ref($t5)
    call $t8 := $ReadRef($t5);
    assert $Event_EventHandle_$invariant_holds($t8);

    // $t8 := Event::emit_event<Config::ConfigChangeEvent<#0>>($t8, $t7)
    call $t8 := $Event_emit_event($Config_ConfigChangeEvent_type_value($tv0), $t8, $t7);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,9686):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t5, $t8)
    call $t5 := $WriteRef($t5, $t8);

    // write_back[Reference($t4)]($t5)
    call $t4 := $WritebackToReference($t5, $t4);

    // write_back[LocalRoot($t2)]($t4)
    call $t2 := $WritebackToValue($t4, 2, $t2);

    // return $t2
    $ret0 := $t2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(2,9899,9):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Config_emit_config_change_event_$direct_intra($tv0: $TypeValue, cap: $Value, value: $Value) returns ($ret0: $Value)
{
    assume $Config_ModifyConfigCapability_$is_well_typed(cap);

    call $ret0 := $Config_emit_config_change_event_$def($tv0, cap, value);
}


procedure {:inline 1} $Config_emit_config_change_event($tv0: $TypeValue, cap: $Value, value: $Value) returns ($ret0: $Value)
{
    assume $Config_ModifyConfigCapability_$is_well_typed(cap);

    call $ret0 := $Config_emit_config_change_event_$def($tv0, cap, value);
}


procedure {:inline 1} $Config_extract_modify_config_capability_$def($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var cap_holder: $Mutation; // ReferenceType($Config_ModifyConfigCapabilityHolder_type_value($tv0))
    var signer_address: $Value; // $AddressType()
    var $t3: $Value; // $AddressType()
    var $t4: $Mutation; // ReferenceType($Option_Option_type_value($Config_ModifyConfigCapability_type_value($tv0)))
    var $t5: $Value; // $Option_Option_type_value($Config_ModifyConfigCapability_type_value($tv0))
    var $t6: $Value; // $Config_ModifyConfigCapability_type_value($tv0)
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(2,7006,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(account)
    call $t3 := $CopyOrMoveValue(account);

    // signer_address := Signer::address_of($t3)
    call signer_address := $Signer_address_of($t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,7204):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // cap_holder := borrow_global<Config::ModifyConfigCapabilityHolder<#0>>(signer_address)
    call cap_holder := $BorrowGlobal($Config_ModifyConfigCapabilityHolder_$memory, signer_address, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1));
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,7250):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref(cap_holder)
    call $Config_ModifyConfigCapabilityHolder_$unpack_ref($tv0, $Dereference(cap_holder));

    // $t4 := borrow_field<Config::ModifyConfigCapabilityHolder<#0>>.cap(cap_holder)
    call $t4 := $BorrowField(cap_holder, $Config_ModifyConfigCapabilityHolder_cap);

    // unpack_ref($t4)
    call $Option_Option_$unpack_ref($Config_ModifyConfigCapability_type_value($tv0), $Dereference($t4));

    // $t5 := read_ref($t4)
    call $t5 := $ReadRef($t4);
    assert $Option_Option_$invariant_holds($t5);

    // ($t6, $t5) := Option::extract<Config::ModifyConfigCapability<#0>>($t5)
    call $t6, $t5 := $Option_extract($Config_ModifyConfigCapability_type_value($tv0), $t5);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,7344):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t4, $t5)
    call $t4 := $WriteRef($t4, $t5);
    if (true) {
     $trace_temp := $Dereference(cap_holder);
      assume {:print "$track_local(2,7344,1):", $trace_temp} true;
    }

    // pack_ref($t4)
    call $Option_Option_$pack_ref($Config_ModifyConfigCapability_type_value($tv0), $Dereference($t4));

    // write_back[Reference(cap_holder)]($t4)
    call cap_holder := $WritebackToReference($t4, cap_holder);

    // pack_ref(cap_holder)
    call $Config_ModifyConfigCapabilityHolder_$pack_ref($tv0, $Dereference(cap_holder));

    // write_back[Config::ModifyConfigCapabilityHolder](cap_holder)
    call $Config_ModifyConfigCapabilityHolder_$memory := $WritebackToGlobal($Config_ModifyConfigCapabilityHolder_$memory, cap_holder);

    // return $t6
    $ret0 := $t6;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(2,7336,7):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Config_extract_modify_config_capability_$direct_inter($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Config_extract_modify_config_capability_$def($tv0, account);
}


procedure {:inline 1} $Config_extract_modify_config_capability_$direct_intra($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Config_extract_modify_config_capability_$def($tv0, account);
}


procedure {:inline 1} $Config_extract_modify_config_capability($tv0: $TypeValue, account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Config_extract_modify_config_capability_$def($tv0, account);
}


procedure {:inline 1} $Config_get_by_address_$def($tv0: $TypeValue, addr: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var tmp#$1: $Value; // $BooleanType()
    var tmp#$2: $Value; // $IntegerType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $BooleanType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $Config_Config_type_value($tv0)
    var $t8: $Value; // $tv0
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := addr;
      assume {:print "$track_local(2,1376,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(addr)
    call $t3 := $CopyOrMoveValue(addr);

    // $t4 := exists<Config::Config<#0>>($t3)
    $t4 := $ResourceExists($Config_Config_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $t3);

    // $t5 := 13
    $t5 := $Integer(13);

    // $t6 := Errors::invalid_state($t5)
    call $t6 := $Errors_invalid_state($t5);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,1529):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t4) goto L0 else goto L1
    if (b#$Boolean($t4)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // abort($t6)
    $abort_code := i#$Integer($t6);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,1479):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // $t7 := get_global<Config::Config<#0>>($t3)
    call $t7 := $GetGlobal($Config_Config_$memory, $t3, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1));
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,1585):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t8 := get_field<Config::Config<#0>>.payload($t7)
    call $t8 := $GetFieldFromValue($t7, $Config_Config_payload);

    // return $t8
    $ret0 := $t8;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(2,1583,9):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Config_get_by_address_$direct_inter($tv0: $TypeValue, addr: $Value) returns ($ret0: $Value)
{
    assume is#$Address(addr);

    call $ret0 := $Config_get_by_address_$def($tv0, addr);
}


procedure {:inline 1} $Config_get_by_address_$direct_intra($tv0: $TypeValue, addr: $Value) returns ($ret0: $Value)
{
    assume is#$Address(addr);

    call $ret0 := $Config_get_by_address_$def($tv0, addr);
}


procedure {:inline 1} $Config_get_by_address($tv0: $TypeValue, addr: $Value) returns ($ret0: $Value)
{
    assume is#$Address(addr);

    call $ret0 := $Config_get_by_address_$def($tv0, addr);
}


procedure {:inline 1} $Config_publish_new_config_$def($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
{
    // declare local variables
    var cap: $Value; // $Config_ModifyConfigCapability_type_value($tv0)
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $tv0
    var $t5: $Value; // $Config_Config_type_value($tv0)
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $Event_EventHandle_type_value($Config_ConfigChangeEvent_type_value($tv0))
    var $t8: $Value; // $Option_Option_type_value($Config_ModifyConfigCapability_type_value($tv0))
    var $t9: $Value; // $Config_ModifyConfigCapabilityHolder_type_value($tv0)
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(2,5207,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := payload;
      assume {:print "$track_local(2,5207,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(account)
    call $t3 := $CopyOrMoveValue(account);

    // $t4 := move(payload)
    call $t4 := $CopyOrMoveValue(payload);

    // $t5 := pack Config::Config<#0>($t4)
    call $t5 := $Config_Config_pack(0, 0, 0, $tv0, $t4);

    // move_to<Config::Config<#0>>($t5, $t3)
    call $Config_Config_$memory := $MoveTo($Config_Config_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $t5, $t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,5310):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t6 := Signer::address_of($t3)
    call $t6 := $Signer_address_of($t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,5440):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t7 := Event::new_event_handle<Config::ConfigChangeEvent<#0>>($t3)
    call $t7 := $Event_new_event_handle($Config_ConfigChangeEvent_type_value($tv0), $t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,5476):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // cap := pack Config::ModifyConfigCapability<#0>($t6, $t7)
    call cap := $Config_ModifyConfigCapability_pack(2, 5378, 2, $tv0, $t6, $t7);
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(2,5378,2):", $trace_temp} true;
    }

    // $t8 := Option::some<Config::ModifyConfigCapability<#0>>(cap)
    call $t8 := $Option_some($Config_ModifyConfigCapability_type_value($tv0), cap);
    if ($abort_flag) {
      goto Abort;
    }

    // $t9 := pack Config::ModifyConfigCapabilityHolder<#0>($t8)
    call $t9 := $Config_ModifyConfigCapabilityHolder_pack(0, 0, 0, $tv0, $t8);

    // move_to<Config::ModifyConfigCapabilityHolder<#0>>($t9, $t3)
    call $Config_ModifyConfigCapabilityHolder_$memory := $MoveTo($Config_ModifyConfigCapabilityHolder_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), $t9, $t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,5544):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Config_publish_new_config_$direct_inter($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
{
    assume is#$Address(account);

    call $Config_publish_new_config_$def($tv0, account, payload);
}


procedure {:inline 1} $Config_publish_new_config_$direct_intra($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
{
    assume is#$Address(account);

    call $Config_publish_new_config_$def($tv0, account, payload);
}


procedure {:inline 1} $Config_publish_new_config($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
{
    assume is#$Address(account);

    call $Config_publish_new_config_$def($tv0, account, payload);
}


procedure {:inline 1} $Config_publish_new_config_with_capability_$def($tv0: $TypeValue, account: $Value, payload: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $tv0
    var $t4: $Value; // $Config_ModifyConfigCapability_type_value($tv0)
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(2,4415,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := payload;
      assume {:print "$track_local(2,4415,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(account)
    call $t2 := $CopyOrMoveValue(account);

    // $t3 := move(payload)
    call $t3 := $CopyOrMoveValue(payload);

    // Config::publish_new_config<#0>($t2, $t3)
    call $Config_publish_new_config($tv0, $t2, $t3);
    if ($abort_flag) {
      goto Abort;
    }

    // $t4 := Config::extract_modify_config_capability<#0>($t2)
    call $t4 := $Config_extract_modify_config_capability($tv0, $t2);
    if ($abort_flag) {
      goto Abort;
    }

    // return $t4
    $ret0 := $t4;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(2,4690,5):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Config_publish_new_config_with_capability_$direct_inter($tv0: $TypeValue, account: $Value, payload: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Config_publish_new_config_with_capability_$def($tv0, account, payload);
}


procedure {:inline 1} $Config_publish_new_config_with_capability_$direct_intra($tv0: $TypeValue, account: $Value, payload: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Config_publish_new_config_with_capability_$def($tv0, account, payload);
}


procedure {:inline 1} $Config_publish_new_config_with_capability($tv0: $TypeValue, account: $Value, payload: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $Config_publish_new_config_with_capability_$def($tv0, account, payload);
}


procedure {:inline 1} $Config_restore_modify_config_capability_$def($tv0: $TypeValue, cap: $Value) returns ()
{
    // declare local variables
    var cap_holder: $Mutation; // ReferenceType($Config_ModifyConfigCapabilityHolder_type_value($tv0))
    var $t2: $Value; // $Config_ModifyConfigCapability_type_value($tv0)
    var $t3: $Value; // $AddressType()
    var $t4: $Mutation; // ReferenceType($Option_Option_type_value($Config_ModifyConfigCapability_type_value($tv0)))
    var $t5: $Value; // $Option_Option_type_value($Config_ModifyConfigCapability_type_value($tv0))
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(2,8148,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(cap)
    call $t2 := $CopyOrMoveValue(cap);

    // $t3 := get_field<Config::ModifyConfigCapability<#0>>.account_address($t2)
    call $t3 := $GetFieldFromValue($t2, $Config_ModifyConfigCapability_account_address);

    // cap_holder := borrow_global<Config::ModifyConfigCapabilityHolder<#0>>($t3)
    call cap_holder := $BorrowGlobal($Config_ModifyConfigCapabilityHolder_$memory, $t3, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1));
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,8321):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref(cap_holder)
    call $Config_ModifyConfigCapabilityHolder_$unpack_ref($tv0, $Dereference(cap_holder));

    // $t4 := borrow_field<Config::ModifyConfigCapabilityHolder<#0>>.cap(cap_holder)
    call $t4 := $BorrowField(cap_holder, $Config_ModifyConfigCapabilityHolder_cap);

    // unpack_ref($t4)
    call $Option_Option_$unpack_ref($Config_ModifyConfigCapability_type_value($tv0), $Dereference($t4));

    // $t5 := read_ref($t4)
    call $t5 := $ReadRef($t4);
    assert $Option_Option_$invariant_holds($t5);

    // $t5 := Option::fill<Config::ModifyConfigCapability<#0>>($t5, $t2)
    call $t5 := $Option_fill($Config_ModifyConfigCapability_type_value($tv0), $t5, $t2);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,8420):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t4, $t5)
    call $t4 := $WriteRef($t4, $t5);
    if (true) {
     $trace_temp := $Dereference(cap_holder);
      assume {:print "$track_local(2,8420,1):", $trace_temp} true;
    }

    // pack_ref($t4)
    call $Option_Option_$pack_ref($Config_ModifyConfigCapability_type_value($tv0), $Dereference($t4));

    // write_back[Reference(cap_holder)]($t4)
    call cap_holder := $WritebackToReference($t4, cap_holder);

    // pack_ref(cap_holder)
    call $Config_ModifyConfigCapabilityHolder_$pack_ref($tv0, $Dereference(cap_holder));

    // write_back[Config::ModifyConfigCapabilityHolder](cap_holder)
    call $Config_ModifyConfigCapabilityHolder_$memory := $WritebackToGlobal($Config_ModifyConfigCapabilityHolder_$memory, cap_holder);

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Config_restore_modify_config_capability_$direct_inter($tv0: $TypeValue, cap: $Value) returns ()
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $Config_restore_modify_config_capability_$def($tv0, cap);
}


procedure {:inline 1} $Config_restore_modify_config_capability_$direct_intra($tv0: $TypeValue, cap: $Value) returns ()
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $Config_restore_modify_config_capability_$def($tv0, cap);
}


procedure {:inline 1} $Config_restore_modify_config_capability($tv0: $TypeValue, cap: $Value) returns ()
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $Config_restore_modify_config_capability_$def($tv0, cap);
}


procedure {:inline 1} $Config_set_$def($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
{
    // declare local variables
    var cap_holder: $Mutation; // ReferenceType($Config_ModifyConfigCapabilityHolder_type_value($tv0))
    var signer_address: $Value; // $AddressType()
    var tmp#$4: $Value; // $BooleanType()
    var tmp#$5: $Value; // $IntegerType()
    var tmp#$6: $Value; // $BooleanType()
    var tmp#$7: $Value; // $IntegerType()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $tv0
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $Option_Option_type_value($Config_ModifyConfigCapability_type_value($tv0))
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Mutation; // ReferenceType($Option_Option_type_value($Config_ModifyConfigCapability_type_value($tv0)))
    var $t18: $Value; // $Option_Option_type_value($Config_ModifyConfigCapability_type_value($tv0))
    var $t19: $Mutation; // ReferenceType($Config_ModifyConfigCapability_type_value($tv0))
    var $t20: $Value; // $Config_ModifyConfigCapability_type_value($tv0)
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(2,2068,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := payload;
      assume {:print "$track_local(2,2068,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t8 := move(account)
    call $t8 := $CopyOrMoveValue(account);

    // $t9 := move(payload)
    call $t9 := $CopyOrMoveValue(payload);

    // signer_address := Signer::address_of($t8)
    call signer_address := $Signer_address_of($t8);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,2229):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t10 := exists<Config::ModifyConfigCapabilityHolder<#0>>(signer_address)
    $t10 := $ResourceExists($Config_ModifyConfigCapabilityHolder_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), signer_address);

    // $t11 := 101
    $t11 := $Integer(101);

    // $t12 := Errors::requires_capability($t11)
    call $t12 := $Errors_requires_capability($t11);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,2388):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t10) goto L0 else goto L1
    if (b#$Boolean($t10)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // abort($t12)
    $abort_code := i#$Integer($t12);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,2306):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // cap_holder := borrow_global<Config::ModifyConfigCapabilityHolder<#0>>(signer_address)
    call cap_holder := $BorrowGlobal($Config_ModifyConfigCapabilityHolder_$memory, signer_address, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1));
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,2466):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref(cap_holder)
    call $Config_ModifyConfigCapabilityHolder_$unpack_ref($tv0, $Dereference(cap_holder));

    // $t13 := get_field<Config::ModifyConfigCapabilityHolder<#0>>.cap(cap_holder)
    call $t13 := $GetFieldFromReference(cap_holder, $Config_ModifyConfigCapabilityHolder_cap);
    assert $Option_Option_$invariant_holds($t13);

    // $t14 := Option::is_some<Config::ModifyConfigCapability<#0>>($t13)
    call $t14 := $Option_is_some($Config_ModifyConfigCapability_type_value($tv0), $t13);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,2567):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t15 := 101
    $t15 := $Integer(101);

    // $t16 := Errors::requires_capability($t15)
    call $t16 := $Errors_requires_capability($t15);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,2601):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t14) goto L2 else goto L3
    if (b#$Boolean($t14)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // destroy(cap_holder)

    // pack_ref(cap_holder)
    call $Config_ModifyConfigCapabilityHolder_$pack_ref($tv0, $Dereference(cap_holder));

    // abort($t16)
    $abort_code := i#$Integer($t16);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,2552):", $trace_abort_temp} true;
    }
    goto Abort;

    // L2:
L2:

    // $t17 := borrow_field<Config::ModifyConfigCapabilityHolder<#0>>.cap(cap_holder)
    call $t17 := $BorrowField(cap_holder, $Config_ModifyConfigCapabilityHolder_cap);

    // unpack_ref_deep($t17)
    call $Option_Option_$unpack_ref_deep($Config_ModifyConfigCapability_type_value($tv0), $Dereference($t17));

    // $t18 := read_ref($t17)
    call $t18 := $ReadRef($t17);
    assert $Option_Option_$invariant_holds($t18);

    // ($t19, $t18) := Option::borrow_mut<Config::ModifyConfigCapability<#0>>($t18)
    call $t19, $t18 := $Option_borrow_mut($Config_ModifyConfigCapability_type_value($tv0), $t18);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,2690):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t17, $t18)
    call $t17 := $WriteRef($t17, $t18);
    if (true) {
     $trace_temp := $Dereference(cap_holder);
      assume {:print "$track_local(2,2690,2):", $trace_temp} true;
    }

    // splice[0 -> $t17]($t19)
    call $t19 := $Splice1(0, $t17, $t19);

    // $t20 := read_ref($t19)
    call $t20 := $ReadRef($t19);
    assert $Config_ModifyConfigCapability_$invariant_holds($t20);

    // $t20 := Config::set_with_capability<#0>($t20, $t9)
    call $t20 := $Config_set_with_capability($tv0, $t20, $t9);
    if ($abort_flag) {
      goto Abort;
    }

    // write_ref($t19, $t20)
    call $t19 := $WriteRef($t19, $t20);
    if (true) {
     $trace_temp := $Dereference(cap_holder);
      assume {:print "$track_local(2,3635,2):", $trace_temp} true;
    }

    // write_back[Reference($t17)]($t19)
    call $t17 := $WritebackToReference($t19, $t17);

    // pack_ref_deep($t17)
    call $Option_Option_$pack_ref_deep($Config_ModifyConfigCapability_type_value($tv0), $Dereference($t17));

    // write_back[Reference(cap_holder)]($t17)
    call cap_holder := $WritebackToReference($t17, cap_holder);

    // pack_ref(cap_holder)
    call $Config_ModifyConfigCapabilityHolder_$pack_ref($tv0, $Dereference(cap_holder));

    // write_back[Config::ModifyConfigCapabilityHolder](cap_holder)
    call $Config_ModifyConfigCapabilityHolder_$memory := $WritebackToGlobal($Config_ModifyConfigCapabilityHolder_$memory, cap_holder);

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Config_set_$direct_inter($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
{
    assume is#$Address(account);

    call $Config_set_$def($tv0, account, payload);
}


procedure {:inline 1} $Config_set_$direct_intra($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
{
    assume is#$Address(account);

    call $Config_set_$def($tv0, account, payload);
}


procedure {:inline 1} $Config_set($tv0: $TypeValue, account: $Value, payload: $Value) returns ()
{
    assume is#$Address(account);

    call $Config_set_$def($tv0, account, payload);
}


procedure {:inline 1} $Config_set_with_capability_$def($tv0: $TypeValue, cap: $Value, payload: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var addr: $Value; // $AddressType()
    var config: $Mutation; // ReferenceType($Config_Config_type_value($tv0))
    var tmp#$4: $Value; // $BooleanType()
    var tmp#$5: $Value; // $IntegerType()
    var $t6: $Value; // $Config_ModifyConfigCapability_type_value($tv0)
    var $t7: $Value; // $tv0
    var $t8: $Mutation; // ReferenceType($Config_ModifyConfigCapability_type_value($tv0))
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $BooleanType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Mutation; // ReferenceType($tv0)
    var $t14: $Value; // $Config_ModifyConfigCapability_type_value($tv0)
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(2,3624,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := payload;
      assume {:print "$track_local(2,3624,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t6 := move(cap)
    call $t6 := $CopyOrMoveValue(cap);

    // $t7 := move(payload)
    call $t7 := $CopyOrMoveValue(payload);

    // $t8 := borrow_local($t6)
    call $t8 := $BorrowLoc(6, $t6);

    // unpack_ref($t8)

    // $t9 := get_field<Config::ModifyConfigCapability<#0>>.account_address($t8)
    call $t9 := $GetFieldFromReference($t8, $Config_ModifyConfigCapability_account_address);

    // addr := $t9
    call addr := $CopyOrMoveValue($t9);
    if (true) {
     $trace_temp := addr;
      assume {:print "$track_local(2,3776,2):", $trace_temp} true;
    }

    // $t10 := exists<Config::Config<#0>>(addr)
    $t10 := $ResourceExists($Config_Config_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1), addr);

    // $t11 := 13
    $t11 := $Integer(13);

    // $t12 := Errors::invalid_state($t11)
    call $t12 := $Errors_invalid_state($t11);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,3862):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t10) goto L0 else goto L1
    if (b#$Boolean($t10)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // destroy($t8)

    // pack_ref($t8)

    // abort($t12)
    $abort_code := i#$Integer($t12);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,3812):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // config := borrow_global<Config::Config<#0>>(addr)
    call config := $BorrowGlobal($Config_Config_$memory, addr, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $tv0], 1));
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(2,3929):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref(config)

    // $t13 := borrow_field<Config::Config<#0>>.payload(config)
    call $t13 := $BorrowField(config, $Config_Config_payload);

    // unpack_ref($t13)

    // write_ref($t13, $t7)
    call $t13 := $WriteRef($t13, $t7);
    if (true) {
     $trace_temp := $Dereference(config);
      assume {:print "$track_local(2,3983,3):", $trace_temp} true;
    }

    // pack_ref($t13)

    // write_back[Reference(config)]($t13)
    call config := $WritebackToReference($t13, config);

    // pack_ref(config)

    // write_back[Config::Config](config)
    call $Config_Config_$memory := $WritebackToGlobal($Config_Config_$memory, config);

    // $t14 := read_ref($t8)
    call $t14 := $ReadRef($t8);
    assert $Config_ModifyConfigCapability_$invariant_holds($t14);

    // $t14 := Config::emit_config_change_event<#0>($t14, $t7)
    call $t14 := $Config_emit_config_change_event($tv0, $t14, $t7);
    if ($abort_flag) {
      goto Abort;
    }

    // write_ref($t8, $t14)
    call $t8 := $WriteRef($t8, $t14);
    if (true) {
     $trace_temp := $Dereference(config);
      assume {:print "$track_local(2,9554,3):", $trace_temp} true;
    }

    // pack_ref($t8)

    // write_back[LocalRoot($t6)]($t8)
    call $t6 := $WritebackToValue($t8, 6, $t6);

    // return $t6
    $ret0 := $t6;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(2,4060,15):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Config_set_with_capability_$direct_inter($tv0: $TypeValue, cap: $Value, payload: $Value) returns ($ret0: $Value)
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $ret0 := $Config_set_with_capability_$def($tv0, cap, payload);
}


procedure {:inline 1} $Config_set_with_capability_$direct_intra($tv0: $TypeValue, cap: $Value, payload: $Value) returns ($ret0: $Value)
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $ret0 := $Config_set_with_capability_$def($tv0, cap, payload);
}


procedure {:inline 1} $Config_set_with_capability($tv0: $TypeValue, cap: $Value, payload: $Value) returns ($ret0: $Value)
{
    assume $Config_ModifyConfigCapability_$is_well_formed(cap);

    call $ret0 := $Config_set_with_capability_$def($tv0, cap, payload);
}




// ** spec vars of module CoreAddresses



// ** spec funs of module CoreAddresses

function {:inline} $CoreAddresses_$GENESIS_ADDRESS(): $Value {
    $Address(1)
}

function {:inline} $CoreAddresses_SPEC_GENESIS_ADDRESS(): $Value {
    $Address(1)
}

function {:inline} $CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS(): $Value {
    $Address(173345816)
}

function {:inline} $CoreAddresses_SPEC_VM_RESERVED_ADDRESS(): $Value {
    $Address(0)
}



// ** structs of module CoreAddresses



// ** functions of module CoreAddresses

procedure {:inline 1} $CoreAddresses_ASSOCIATION_ROOT_ADDRESS_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0xa550c18
    $t0 := $Address(173345816);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(3,1298,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_ASSOCIATION_ROOT_ADDRESS_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_ASSOCIATION_ROOT_ADDRESS_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_ASSOCIATION_ROOT_ADDRESS() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_ASSOCIATION_ROOT_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_GENESIS_ADDRESS_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0x1
    $t0 := $Address(1);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(3,294,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_GENESIS_ADDRESS_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_GENESIS_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_GENESIS_ADDRESS_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_GENESIS_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_GENESIS_ADDRESS() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_GENESIS_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_VM_RESERVED_ADDRESS_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0x0
    $t0 := $Address(0);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(3,1828,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $CoreAddresses_VM_RESERVED_ADDRESS_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_VM_RESERVED_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_VM_RESERVED_ADDRESS_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_VM_RESERVED_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_VM_RESERVED_ADDRESS() returns ($ret0: $Value)
{
    call $ret0 := $CoreAddresses_VM_RESERVED_ADDRESS_$def();
}


procedure {:inline 1} $CoreAddresses_assert_genesis_address_$def(account: $Value) returns ()
{
    // declare local variables
    var tmp#$1: $Value; // $BooleanType()
    var tmp#$2: $Value; // $IntegerType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $BooleanType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(3,467,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(account)
    call $t3 := $CopyOrMoveValue(account);

    // $t4 := Signer::address_of($t3)
    call $t4 := $Signer_address_of($t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(3,544):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t5 := CoreAddresses::GENESIS_ADDRESS()
    call $t5 := $CoreAddresses_GENESIS_ADDRESS();
    if ($abort_flag) {
      goto Abort;
    }

    // $t6 := ==($t4, $t5)
    $t6 := $Boolean($IsEqual($t4, $t5));

    // $t7 := 11
    $t7 := $Integer(11);

    // $t8 := Errors::requires_address($t7)
    call $t8 := $Errors_requires_address($t7);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(3,594):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t6) goto L0 else goto L1
    if (b#$Boolean($t6)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // abort($t8)
    $abort_code := i#$Integer($t8);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(3,529):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $CoreAddresses_assert_genesis_address_$direct_inter(account: $Value) returns ()
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(!$IsEqual($Signer_spec_address_of(account), $CoreAddresses_SPEC_GENESIS_ADDRESS())))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!$IsEqual($Signer_spec_address_of(account), $CoreAddresses_SPEC_GENESIS_ADDRESS())))));

procedure {:inline 1} $CoreAddresses_assert_genesis_address_$direct_intra(account: $Value) returns ()
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(!$IsEqual($Signer_spec_address_of(account), $CoreAddresses_SPEC_GENESIS_ADDRESS())))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!$IsEqual($Signer_spec_address_of(account), $CoreAddresses_SPEC_GENESIS_ADDRESS())))));

procedure {:inline 1} $CoreAddresses_assert_genesis_address(account: $Value) returns ()
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(!$IsEqual($Signer_spec_address_of(account), $CoreAddresses_SPEC_GENESIS_ADDRESS())))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!$IsEqual($Signer_spec_address_of(account), $CoreAddresses_SPEC_GENESIS_ADDRESS())))));



// ** spec vars of module Version



// ** spec funs of module Version



// ** structs of module Version

const unique $Version_Version: $TypeName;
const $Version_Version_major: $FieldName;
axiom $Version_Version_major == 0;
function $Version_Version_type_value(): $TypeValue {
    $StructType($Version_Version, $EmptyTypeValueArray)
}
var $Version_Version_$memory: $Memory;
var $Version_Version_$memory_$old: $Memory;
function {:inline} $Version_Version_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $Version_Version_major))
}
function {:inline} $Version_Version_$invariant_holds($this: $Value): bool {
    true
}

function {:inline} $Version_Version_$is_well_formed($this: $Value): bool {
    $Version_Version_$is_well_typed($this) && $Version_Version_$invariant_holds($this)}

procedure {:inline 1} $Version_Version_pack($file_id: int, $byte_index: int, $var_idx: int, major: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(major);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := major], 1));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $Version_Version_unpack($struct: $Value) returns (major: $Value)
{
    assume is#$Vector($struct);
    major := $SelectField($struct, $Version_Version_major);
    assume $IsValidU64(major);
}



// ** functions of module Version

procedure {:inline 1} $Version_get_$def(addr: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var version: $Value; // $Version_Version_type_value()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := addr;
      assume {:print "$track_local(12,368,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(addr)
    call $t2 := $CopyOrMoveValue(addr);

    // version := Config::get_by_address<Version::Version>($t2)
    call version := $Config_get_by_address($Version_Version_type_value(), $t2);
    if ($abort_flag) {
      goto Abort;
    }

    // $t3 := get_field<Version::Version>.major(version)
    call $t3 := $GetFieldFromValue(version, $Version_Version_major);

    // return $t3
    $ret0 := $t3;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(12,480,4):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Version_get_$direct_inter(addr: $Value) returns ($ret0: $Value)
{
    assume is#$Address(addr);

    call $ret0 := $Version_get_$def(addr);
}


procedure {:inline 1} $Version_get_$direct_intra(addr: $Value) returns ($ret0: $Value)
{
    assume is#$Address(addr);

    call $ret0 := $Version_get_$def(addr);
}


procedure {:inline 1} $Version_get(addr: $Value) returns ($ret0: $Value)
{
    assume is#$Address(addr);

    call $ret0 := $Version_get_$def(addr);
}


procedure {:inline 1} $Version_new_version_$def(major: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $Version_Version_type_value()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := major;
      assume {:print "$track_local(12,226,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(major)
    call $t1 := $CopyOrMoveValue(major);

    // $t2 := pack Version::Version($t1)
    call $t2 := $Version_Version_pack(0, 0, 0, $t1);

    // return $t2
    $ret0 := $t2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(12,280,3):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Version_new_version_$direct_inter(major: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(major);

    call $ret0 := $Version_new_version_$def(major);
}


procedure {:inline 1} $Version_new_version_$direct_intra(major: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(major);

    call $ret0 := $Version_new_version_$def(major);
}


procedure {:inline 1} $Version_new_version(major: $Value) returns ($ret0: $Value)
{
    assume $IsValidU64(major);

    call $ret0 := $Version_new_version_$def(major);
}




// ** spec vars of module Timestamp



// ** spec funs of module Timestamp

function {:inline} $Timestamp_$is_genesis($Timestamp_TimeHasStarted_$memory: $Memory): $Value {
    $Boolean(!b#$Boolean($ResourceExists($Timestamp_TimeHasStarted_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS())))
}

function {:inline} $Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory: $Memory): $Value {
    $SelectField($ResourceValue($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS()), $Timestamp_CurrentTimeMilliseconds_milliseconds)
}

function {:inline} $Timestamp_spec_now_seconds($Timestamp_CurrentTimeMilliseconds_$memory: $Memory): $Value {
    $Integer(i#$Integer($SelectField($ResourceValue($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_SPEC_GENESIS_ADDRESS()), $Timestamp_CurrentTimeMilliseconds_milliseconds)) div i#$Integer($Integer(1000)))
}

function {:inline} $Timestamp_spec_now_millseconds($Timestamp_CurrentTimeMilliseconds_$memory: $Memory): $Value {
    $SelectField($ResourceValue($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_SPEC_GENESIS_ADDRESS()), $Timestamp_CurrentTimeMilliseconds_milliseconds)
}



// ** structs of module Timestamp

const unique $Timestamp_CurrentTimeMilliseconds: $TypeName;
const $Timestamp_CurrentTimeMilliseconds_milliseconds: $FieldName;
axiom $Timestamp_CurrentTimeMilliseconds_milliseconds == 0;
function $Timestamp_CurrentTimeMilliseconds_type_value(): $TypeValue {
    $StructType($Timestamp_CurrentTimeMilliseconds, $EmptyTypeValueArray)
}
var $Timestamp_CurrentTimeMilliseconds_$memory: $Memory;
var $Timestamp_CurrentTimeMilliseconds_$memory_$old: $Memory;
function {:inline} $Timestamp_CurrentTimeMilliseconds_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU64($SelectField($this, $Timestamp_CurrentTimeMilliseconds_milliseconds))
}
function {:inline} $Timestamp_CurrentTimeMilliseconds_$invariant_holds($this: $Value): bool {
    true
}

function {:inline} $Timestamp_CurrentTimeMilliseconds_$is_well_formed($this: $Value): bool {
    $Timestamp_CurrentTimeMilliseconds_$is_well_typed($this) && $Timestamp_CurrentTimeMilliseconds_$invariant_holds($this)}

procedure {:inline 1} $Timestamp_CurrentTimeMilliseconds_pack($file_id: int, $byte_index: int, $var_idx: int, milliseconds: $Value) returns ($struct: $Value)
{
    assume $IsValidU64(milliseconds);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := milliseconds], 1));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $Timestamp_CurrentTimeMilliseconds_unpack($struct: $Value) returns (milliseconds: $Value)
{
    assume is#$Vector($struct);
    milliseconds := $SelectField($struct, $Timestamp_CurrentTimeMilliseconds_milliseconds);
    assume $IsValidU64(milliseconds);
}

const unique $Timestamp_TimeHasStarted: $TypeName;
const $Timestamp_TimeHasStarted_dummy_field: $FieldName;
axiom $Timestamp_TimeHasStarted_dummy_field == 0;
function $Timestamp_TimeHasStarted_type_value(): $TypeValue {
    $StructType($Timestamp_TimeHasStarted, $EmptyTypeValueArray)
}
var $Timestamp_TimeHasStarted_$memory: $Memory;
var $Timestamp_TimeHasStarted_$memory_$old: $Memory;
function {:inline} $Timestamp_TimeHasStarted_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 1
      && is#$Boolean($SelectField($this, $Timestamp_TimeHasStarted_dummy_field))
}
function {:inline} $Timestamp_TimeHasStarted_$invariant_holds($this: $Value): bool {
    true
}

function {:inline} $Timestamp_TimeHasStarted_$is_well_formed($this: $Value): bool {
    $Timestamp_TimeHasStarted_$is_well_typed($this) && $Timestamp_TimeHasStarted_$invariant_holds($this)}

procedure {:inline 1} $Timestamp_TimeHasStarted_pack($file_id: int, $byte_index: int, $var_idx: int, dummy_field: $Value) returns ($struct: $Value)
{
    assume is#$Boolean(dummy_field);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := dummy_field], 1));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $Timestamp_TimeHasStarted_unpack($struct: $Value) returns (dummy_field: $Value)
{
    assume is#$Vector($struct);
    dummy_field := $SelectField($struct, $Timestamp_TimeHasStarted_dummy_field);
    assume is#$Boolean(dummy_field);
}



// ** functions of module Timestamp

procedure {:inline 1} $Timestamp_assert_genesis_$def() returns ()
{
    // declare local variables
    var tmp#$0: $Value; // $BooleanType()
    var tmp#$1: $Value; // $IntegerType()
    var $t2: $Value; // $BooleanType()
    var $t3: $Value; // $IntegerType()
    var $t4: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t2 := Timestamp::is_genesis()
    call $t2 := $Timestamp_is_genesis();
    if ($abort_flag) {
      goto Abort;
    }

    // $t3 := 12
    $t3 := $Integer(12);

    // $t4 := Errors::invalid_state($t3)
    call $t4 := $Errors_invalid_state($t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,4920):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t2) goto L0 else goto L1
    if (b#$Boolean($t2)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // abort($t4)
    $abort_code := i#$Integer($t4);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,4891):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Timestamp_assert_genesis_$direct_inter() returns ()
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(!b#$Boolean($Timestamp_$is_genesis($Timestamp_TimeHasStarted_$memory))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!b#$Boolean($Timestamp_$is_genesis($Timestamp_TimeHasStarted_$memory))))));

procedure {:inline 1} $Timestamp_assert_genesis_$direct_intra() returns ()
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(!b#$Boolean($Timestamp_$is_genesis($Timestamp_TimeHasStarted_$memory))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!b#$Boolean($Timestamp_$is_genesis($Timestamp_TimeHasStarted_$memory))))));

procedure {:inline 1} $Timestamp_assert_genesis() returns ()
;
modifies $abort_flag, $abort_code;
ensures b#$Boolean(old($Boolean(!b#$Boolean($Timestamp_$is_genesis($Timestamp_TimeHasStarted_$memory))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!b#$Boolean($Timestamp_$is_genesis($Timestamp_TimeHasStarted_$memory))))));

procedure {:inline 1} $Timestamp_initialize_$def(account: $Value, genesis_timestamp: $Value) returns ()
{
    // declare local variables
    var milli_timer: $Value; // $Timestamp_CurrentTimeMilliseconds_type_value()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(10,781,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := genesis_timestamp;
      assume {:print "$track_local(10,781,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(account)
    call $t3 := $CopyOrMoveValue(account);

    // $t4 := move(genesis_timestamp)
    call $t4 := $CopyOrMoveValue(genesis_timestamp);

    // CoreAddresses::assert_genesis_address($t3)
    call $CoreAddresses_assert_genesis_address($t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,918):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // milli_timer := pack Timestamp::CurrentTimeMilliseconds($t4)
    call milli_timer := $Timestamp_CurrentTimeMilliseconds_pack(10, 977, 2, $t4);
    if (true) {
     $trace_temp := milli_timer;
      assume {:print "$track_local(10,977,2):", $trace_temp} true;
    }

    // move_to<Timestamp::CurrentTimeMilliseconds>(milli_timer, $t3)
    call $Timestamp_CurrentTimeMilliseconds_$memory := $MoveTo($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, milli_timer, $t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,1044):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Timestamp_initialize_$direct_inter(account: $Value, genesis_timestamp: $Value) returns ()
{
    assume is#$Address(account);

    assume $IsValidU64(genesis_timestamp);

    call $Timestamp_initialize_$def(account, genesis_timestamp);
}


procedure {:inline 1} $Timestamp_initialize_$direct_intra(account: $Value, genesis_timestamp: $Value) returns ()
{
    assume is#$Address(account);

    assume $IsValidU64(genesis_timestamp);

    call $Timestamp_initialize_$def(account, genesis_timestamp);
}


procedure {:inline 1} $Timestamp_initialize(account: $Value, genesis_timestamp: $Value) returns ()
{
    assume is#$Address(account);

    assume $IsValidU64(genesis_timestamp);

    call $Timestamp_initialize_$def(account, genesis_timestamp);
}


procedure {:inline 1} $Timestamp_is_genesis_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $BooleanType()
    var $t2: $Value; // $BooleanType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::GENESIS_ADDRESS()
    call $t0 := $CoreAddresses_GENESIS_ADDRESS();
    if ($abort_flag) {
      goto Abort;
    }

    // $t1 := exists<Timestamp::TimeHasStarted>($t0)
    $t1 := $ResourceExists($Timestamp_TimeHasStarted_$memory, $EmptyTypeValueArray, $t0);

    // $t2 := !($t1)
    call $t2 := $Not($t1);

    // return $t2
    $ret0 := $t2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(10,4587,3):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Timestamp_is_genesis_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $Timestamp_is_genesis_$def();
}


procedure {:inline 1} $Timestamp_is_genesis_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $Timestamp_is_genesis_$def();
}


procedure {:inline 1} $Timestamp_is_genesis() returns ($ret0: $Value)
{
    call $ret0 := $Timestamp_is_genesis_$def();
}


procedure {:inline 1} $Timestamp_now_milliseconds_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $AddressType()
    var $t1: $Value; // $Timestamp_CurrentTimeMilliseconds_type_value()
    var $t2: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := CoreAddresses::GENESIS_ADDRESS()
    call $t0 := $CoreAddresses_GENESIS_ADDRESS();
    if ($abort_flag) {
      goto Abort;
    }

    // $t1 := get_global<Timestamp::CurrentTimeMilliseconds>($t0)
    call $t1 := $GetGlobal($Timestamp_CurrentTimeMilliseconds_$memory, $t0, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,3142):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t2 := get_field<Timestamp::CurrentTimeMilliseconds>.milliseconds($t1)
    call $t2 := $GetFieldFromValue($t1, $Timestamp_CurrentTimeMilliseconds_milliseconds);

    // return $t2
    $ret0 := $t2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(10,3142,3):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Timestamp_now_milliseconds_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $Timestamp_now_milliseconds_$def();
}


procedure {:inline 1} $Timestamp_now_milliseconds_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $Timestamp_now_milliseconds_$def();
}


procedure {:inline 1} $Timestamp_now_milliseconds() returns ($ret0: $Value)
{
    call $ret0 := $Timestamp_now_milliseconds_$def();
}


procedure {:inline 1} $Timestamp_now_seconds_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $t1: $Value; // $IntegerType()
    var $t2: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := Timestamp::now_milliseconds()
    call $t0 := $Timestamp_now_milliseconds();
    if ($abort_flag) {
      goto Abort;
    }

    // $t1 := 1000
    $t1 := $Integer(1000);

    // $t2 := /($t0, $t1)
    call $t2 := $Div($t0, $t1);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,2604):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t2
    $ret0 := $t2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(10,2585,3):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $Timestamp_now_seconds_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $Timestamp_now_seconds_$def();
}


procedure {:inline 1} $Timestamp_now_seconds_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $Timestamp_now_seconds_$def();
}


procedure {:inline 1} $Timestamp_now_seconds() returns ($ret0: $Value)
{
    call $ret0 := $Timestamp_now_seconds_$def();
}


procedure {:inline 1} $Timestamp_set_time_has_started_$def(account: $Value) returns ()
{
    // declare local variables
    var tmp#$1: $Value; // $BooleanType()
    var tmp#$2: $Value; // $IntegerType()
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $IntegerType()
    var $t8: $Value; // $BooleanType()
    var $t9: $Value; // $Timestamp_TimeHasStarted_type_value()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(10,3725,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(account)
    call $t3 := $CopyOrMoveValue(account);

    // CoreAddresses::assert_genesis_address($t3)
    call $CoreAddresses_assert_genesis_address($t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,3800):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t4 := CoreAddresses::GENESIS_ADDRESS()
    call $t4 := $CoreAddresses_GENESIS_ADDRESS();
    if ($abort_flag) {
      goto Abort;
    }

    // $t5 := exists<Timestamp::CurrentTimeMilliseconds>($t4)
    $t5 := $ResourceExists($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $t4);

    // $t6 := 101
    $t6 := $Integer(101);

    // $t7 := Errors::invalid_state($t6)
    call $t7 := $Errors_invalid_state($t6);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,4001):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t5) goto L0 else goto L1
    if (b#$Boolean($t5)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // destroy($t3)

    // abort($t7)
    $abort_code := i#$Integer($t7);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,3894):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // $t8 := false
    $t8 := $Boolean(false);

    // $t9 := pack Timestamp::TimeHasStarted($t8)
    call $t9 := $Timestamp_TimeHasStarted_pack(0, 0, 0, $t8);

    // move_to<Timestamp::TimeHasStarted>($t9, $t3)
    call $Timestamp_TimeHasStarted_$memory := $MoveTo($Timestamp_TimeHasStarted_$memory, $EmptyTypeValueArray, $t9, $t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,4052):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Timestamp_set_time_has_started_$direct_inter(account: $Value) returns ()
{
    assume is#$Address(account);

    call $Timestamp_set_time_has_started_$def(account);
}


procedure {:inline 1} $Timestamp_set_time_has_started_$direct_intra(account: $Value) returns ()
{
    assume is#$Address(account);

    call $Timestamp_set_time_has_started_$def(account);
}


procedure {:inline 1} $Timestamp_set_time_has_started(account: $Value) returns ()
{
    assume is#$Address(account);

    call $Timestamp_set_time_has_started_$def(account);
}


procedure {:inline 1} $Timestamp_update_global_time_$def(account: $Value, timestamp: $Value) returns ()
{
    // declare local variables
    var global_milli_timer: $Mutation; // ReferenceType($Timestamp_CurrentTimeMilliseconds_type_value())
    var tmp#$3: $Value; // $BooleanType()
    var tmp#$4: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $AddressType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $BooleanType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $IntegerType()
    var $t12: $Mutation; // ReferenceType($IntegerType())
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(10,1517,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := timestamp;
      assume {:print "$track_local(10,1517,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t5 := move(account)
    call $t5 := $CopyOrMoveValue(account);

    // $t6 := move(timestamp)
    call $t6 := $CopyOrMoveValue(timestamp);

    // CoreAddresses::assert_genesis_address($t5)
    call $CoreAddresses_assert_genesis_address($t5);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,1639):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t7 := CoreAddresses::GENESIS_ADDRESS()
    call $t7 := $CoreAddresses_GENESIS_ADDRESS();
    if ($abort_flag) {
      goto Abort;
    }

    // global_milli_timer := borrow_global<Timestamp::CurrentTimeMilliseconds>($t7)
    call global_milli_timer := $BorrowGlobal($Timestamp_CurrentTimeMilliseconds_$memory, $t7, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,1753):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref(global_milli_timer)

    // $t8 := get_field<Timestamp::CurrentTimeMilliseconds>.milliseconds(global_milli_timer)
    call $t8 := $GetFieldFromReference(global_milli_timer, $Timestamp_CurrentTimeMilliseconds_milliseconds);

    // $t9 := >($t6, $t8)
    call $t9 := $Gt($t6, $t8);

    // $t10 := 14
    $t10 := $Integer(14);

    // $t11 := Errors::invalid_argument($t10)
    call $t11 := $Errors_invalid_argument($t10);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,1899):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t9) goto L0 else goto L1
    if (b#$Boolean($t9)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // destroy(global_milli_timer)

    // pack_ref(global_milli_timer)

    // abort($t11)
    $abort_code := i#$Integer($t11);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(10,1839):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // $t12 := borrow_field<Timestamp::CurrentTimeMilliseconds>.milliseconds(global_milli_timer)
    call $t12 := $BorrowField(global_milli_timer, $Timestamp_CurrentTimeMilliseconds_milliseconds);

    // unpack_ref($t12)

    // write_ref($t12, $t6)
    call $t12 := $WriteRef($t12, $t6);
    if (true) {
     $trace_temp := $Dereference(global_milli_timer);
      assume {:print "$track_local(10,1946,2):", $trace_temp} true;
    }

    // pack_ref($t12)

    // write_back[Reference(global_milli_timer)]($t12)
    call global_milli_timer := $WritebackToReference($t12, global_milli_timer);

    // pack_ref(global_milli_timer)

    // write_back[Timestamp::CurrentTimeMilliseconds](global_milli_timer)
    call $Timestamp_CurrentTimeMilliseconds_$memory := $WritebackToGlobal($Timestamp_CurrentTimeMilliseconds_$memory, global_milli_timer);

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $Timestamp_update_global_time_$direct_inter(account: $Value, timestamp: $Value) returns ()
{
    assume is#$Address(account);

    assume $IsValidU64(timestamp);

    call $Timestamp_update_global_time_$def(account, timestamp);
}


procedure {:inline 1} $Timestamp_update_global_time_$direct_intra(account: $Value, timestamp: $Value) returns ()
{
    assume is#$Address(account);

    assume $IsValidU64(timestamp);

    call $Timestamp_update_global_time_$def(account, timestamp);
}


procedure {:inline 1} $Timestamp_update_global_time(account: $Value, timestamp: $Value) returns ()
{
    assume is#$Address(account);

    assume $IsValidU64(timestamp);

    call $Timestamp_update_global_time_$def(account, timestamp);
}




// ** spec vars of module PackageTxnManager



// ** spec funs of module PackageTxnManager

function {:inline} $PackageTxnManager_$get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory: $Memory, module_address: $Value): $Value {
    if (b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, module_address))) then ($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, module_address), $PackageTxnManager_ModuleUpgradeStrategy_strategy)) else ($Integer(0))
}

function {:inline} $PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory: $Memory, module_address: $Value): $Value {
    if (b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, module_address))) then ($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, module_address), $PackageTxnManager_ModuleUpgradeStrategy_strategy)) else ($Integer(0))
}

function {:inline} $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory: $Memory, module_address: $Value): $Value {
    if (b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, module_address))) then ($SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, module_address), $PackageTxnManager_TwoPhaseUpgrade_plan)) else ($Option_spec_none($PackageTxnManager_UpgradePlan_type_value()))
}

function {:inline} $PackageTxnManager_holder$21($Config_ModifyConfigCapabilityHolder_$memory: $Memory, account: $Value): $Value {
    $ResourceValue($Config_ModifyConfigCapabilityHolder_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Version_Version_type_value()], 1), $Signer_$address_of(account))
}

function {:inline} $PackageTxnManager_active_after_time$22($Timestamp_CurrentTimeMilliseconds_$memory: $Memory, min_milliseconds: $Value): $Value {
    $Integer(i#$Integer($Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory)) + i#$Integer(min_milliseconds))
}

function {:inline} $PackageTxnManager_tpu$23($PackageTxnManager_TwoPhaseUpgrade_$memory: $Memory, package_address: $Value): $Value {
    $ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, package_address)
}



// ** structs of module PackageTxnManager

const unique $PackageTxnManager_ModuleUpgradeStrategy: $TypeName;
const $PackageTxnManager_ModuleUpgradeStrategy_strategy: $FieldName;
axiom $PackageTxnManager_ModuleUpgradeStrategy_strategy == 0;
function $PackageTxnManager_ModuleUpgradeStrategy_type_value(): $TypeValue {
    $StructType($PackageTxnManager_ModuleUpgradeStrategy, $EmptyTypeValueArray)
}
var $PackageTxnManager_ModuleUpgradeStrategy_$memory: $Memory;
var $PackageTxnManager_ModuleUpgradeStrategy_$memory_$old: $Memory;
function {:inline} $PackageTxnManager_ModuleUpgradeStrategy_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 1
      && $IsValidU8($SelectField($this, $PackageTxnManager_ModuleUpgradeStrategy_strategy))
}
function {:inline} $PackageTxnManager_ModuleUpgradeStrategy_$invariant_holds($this: $Value): bool {
    true
}

function {:inline} $PackageTxnManager_ModuleUpgradeStrategy_$is_well_formed($this: $Value): bool {
    $PackageTxnManager_ModuleUpgradeStrategy_$is_well_typed($this) && $PackageTxnManager_ModuleUpgradeStrategy_$invariant_holds($this)}

procedure {:inline 1} $PackageTxnManager_ModuleUpgradeStrategy_pack($file_id: int, $byte_index: int, $var_idx: int, strategy: $Value) returns ($struct: $Value)
{
    assume $IsValidU8(strategy);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := strategy], 1));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $PackageTxnManager_ModuleUpgradeStrategy_unpack($struct: $Value) returns (strategy: $Value)
{
    assume is#$Vector($struct);
    strategy := $SelectField($struct, $PackageTxnManager_ModuleUpgradeStrategy_strategy);
    assume $IsValidU8(strategy);
}

const unique $PackageTxnManager_TwoPhaseUpgrade: $TypeName;
const $PackageTxnManager_TwoPhaseUpgrade_plan: $FieldName;
axiom $PackageTxnManager_TwoPhaseUpgrade_plan == 0;
const $PackageTxnManager_TwoPhaseUpgrade_version_cap: $FieldName;
axiom $PackageTxnManager_TwoPhaseUpgrade_version_cap == 1;
const $PackageTxnManager_TwoPhaseUpgrade_upgrade_event: $FieldName;
axiom $PackageTxnManager_TwoPhaseUpgrade_upgrade_event == 2;
function $PackageTxnManager_TwoPhaseUpgrade_type_value(): $TypeValue {
    $StructType($PackageTxnManager_TwoPhaseUpgrade, $EmptyTypeValueArray)
}
var $PackageTxnManager_TwoPhaseUpgrade_$memory: $Memory;
var $PackageTxnManager_TwoPhaseUpgrade_$memory_$old: $Memory;
function {:inline} $PackageTxnManager_TwoPhaseUpgrade_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 3
      && $Option_Option_$is_well_typed($SelectField($this, $PackageTxnManager_TwoPhaseUpgrade_plan))
      && $Config_ModifyConfigCapability_$is_well_typed($SelectField($this, $PackageTxnManager_TwoPhaseUpgrade_version_cap))
      && $Event_EventHandle_$is_well_typed($SelectField($this, $PackageTxnManager_TwoPhaseUpgrade_upgrade_event))
}
function {:inline} $PackageTxnManager_TwoPhaseUpgrade_$invariant_holds($this: $Value): bool {
    $Option_Option_$invariant_holds($SelectField($this, $PackageTxnManager_TwoPhaseUpgrade_plan))
      && $Config_ModifyConfigCapability_$invariant_holds($SelectField($this, $PackageTxnManager_TwoPhaseUpgrade_version_cap))
      && $Event_EventHandle_$invariant_holds($SelectField($this, $PackageTxnManager_TwoPhaseUpgrade_upgrade_event))
}

function {:inline} $PackageTxnManager_TwoPhaseUpgrade_$is_well_formed($this: $Value): bool {
    $PackageTxnManager_TwoPhaseUpgrade_$is_well_typed($this) && $PackageTxnManager_TwoPhaseUpgrade_$invariant_holds($this)}

procedure {:inline 1} $PackageTxnManager_TwoPhaseUpgrade_$unpack_ref_deep($before: $Value) {
    call $Option_Option_$unpack_ref($PackageTxnManager_UpgradePlan_type_value(), $SelectField($before, $PackageTxnManager_TwoPhaseUpgrade_plan));
    assume $PackageTxnManager_TwoPhaseUpgrade_$invariant_holds($before);
}

procedure {:inline 1} $PackageTxnManager_TwoPhaseUpgrade_$unpack_ref($before: $Value) {
    assume $PackageTxnManager_TwoPhaseUpgrade_$invariant_holds($before);
}

procedure {:inline 1} $PackageTxnManager_TwoPhaseUpgrade_$pack_ref_deep($after: $Value) {
    call $Option_Option_$pack_ref($PackageTxnManager_UpgradePlan_type_value(), $SelectField($after, $PackageTxnManager_TwoPhaseUpgrade_plan));
}

procedure {:inline 1} $PackageTxnManager_TwoPhaseUpgrade_$pack_ref($after: $Value) {
}

procedure {:inline 1} $PackageTxnManager_TwoPhaseUpgrade_pack($file_id: int, $byte_index: int, $var_idx: int, plan: $Value, version_cap: $Value, upgrade_event: $Value) returns ($struct: $Value)
{
    assume $Option_Option_$is_well_formed(plan);
    assume $Config_ModifyConfigCapability_$is_well_formed(version_cap);
    assume $Event_EventHandle_$is_well_formed(upgrade_event);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := plan][1 := version_cap][2 := upgrade_event], 3));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $PackageTxnManager_TwoPhaseUpgrade_unpack($struct: $Value) returns (plan: $Value, version_cap: $Value, upgrade_event: $Value)
{
    assume is#$Vector($struct);
    plan := $SelectField($struct, $PackageTxnManager_TwoPhaseUpgrade_plan);
    assume $Option_Option_$is_well_formed(plan);
    version_cap := $SelectField($struct, $PackageTxnManager_TwoPhaseUpgrade_version_cap);
    assume $Config_ModifyConfigCapability_$is_well_formed(version_cap);
    upgrade_event := $SelectField($struct, $PackageTxnManager_TwoPhaseUpgrade_upgrade_event);
    assume $Event_EventHandle_$is_well_formed(upgrade_event);
}

const unique $PackageTxnManager_UpgradeEvent: $TypeName;
const $PackageTxnManager_UpgradeEvent_package_address: $FieldName;
axiom $PackageTxnManager_UpgradeEvent_package_address == 0;
const $PackageTxnManager_UpgradeEvent_package_hash: $FieldName;
axiom $PackageTxnManager_UpgradeEvent_package_hash == 1;
const $PackageTxnManager_UpgradeEvent_version: $FieldName;
axiom $PackageTxnManager_UpgradeEvent_version == 2;
function $PackageTxnManager_UpgradeEvent_type_value(): $TypeValue {
    $StructType($PackageTxnManager_UpgradeEvent, $EmptyTypeValueArray)
}
var $PackageTxnManager_UpgradeEvent_$memory: $Memory;
var $PackageTxnManager_UpgradeEvent_$memory_$old: $Memory;
function {:inline} $PackageTxnManager_UpgradeEvent_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 3
      && is#$Address($SelectField($this, $PackageTxnManager_UpgradeEvent_package_address))
      && $Vector_$is_well_formed($SelectField($this, $PackageTxnManager_UpgradeEvent_package_hash)) && (forall $$0: int :: {$select_vector($SelectField($this, $PackageTxnManager_UpgradeEvent_package_hash),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $PackageTxnManager_UpgradeEvent_package_hash)) ==> $IsValidU8($select_vector($SelectField($this, $PackageTxnManager_UpgradeEvent_package_hash),$$0)))
      && $IsValidU64($SelectField($this, $PackageTxnManager_UpgradeEvent_version))
}
function {:inline} $PackageTxnManager_UpgradeEvent_$invariant_holds($this: $Value): bool {
    true
}

function {:inline} $PackageTxnManager_UpgradeEvent_$is_well_formed($this: $Value): bool {
    $PackageTxnManager_UpgradeEvent_$is_well_typed($this) && $PackageTxnManager_UpgradeEvent_$invariant_holds($this)}

procedure {:inline 1} $PackageTxnManager_UpgradeEvent_pack($file_id: int, $byte_index: int, $var_idx: int, package_address: $Value, package_hash: $Value, version: $Value) returns ($struct: $Value)
{
    assume is#$Address(package_address);
    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));
    assume $IsValidU64(version);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := package_address][1 := package_hash][2 := version], 3));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $PackageTxnManager_UpgradeEvent_unpack($struct: $Value) returns (package_address: $Value, package_hash: $Value, version: $Value)
{
    assume is#$Vector($struct);
    package_address := $SelectField($struct, $PackageTxnManager_UpgradeEvent_package_address);
    assume is#$Address(package_address);
    package_hash := $SelectField($struct, $PackageTxnManager_UpgradeEvent_package_hash);
    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));
    version := $SelectField($struct, $PackageTxnManager_UpgradeEvent_version);
    assume $IsValidU64(version);
}

const unique $PackageTxnManager_UpgradePlan: $TypeName;
const $PackageTxnManager_UpgradePlan_package_hash: $FieldName;
axiom $PackageTxnManager_UpgradePlan_package_hash == 0;
const $PackageTxnManager_UpgradePlan_active_after_time: $FieldName;
axiom $PackageTxnManager_UpgradePlan_active_after_time == 1;
const $PackageTxnManager_UpgradePlan_version: $FieldName;
axiom $PackageTxnManager_UpgradePlan_version == 2;
function $PackageTxnManager_UpgradePlan_type_value(): $TypeValue {
    $StructType($PackageTxnManager_UpgradePlan, $EmptyTypeValueArray)
}
var $PackageTxnManager_UpgradePlan_$memory: $Memory;
var $PackageTxnManager_UpgradePlan_$memory_$old: $Memory;
function {:inline} $PackageTxnManager_UpgradePlan_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 3
      && $Vector_$is_well_formed($SelectField($this, $PackageTxnManager_UpgradePlan_package_hash)) && (forall $$0: int :: {$select_vector($SelectField($this, $PackageTxnManager_UpgradePlan_package_hash),$$0)} $$0 >= 0 && $$0 < $vlen($SelectField($this, $PackageTxnManager_UpgradePlan_package_hash)) ==> $IsValidU8($select_vector($SelectField($this, $PackageTxnManager_UpgradePlan_package_hash),$$0)))
      && $IsValidU64($SelectField($this, $PackageTxnManager_UpgradePlan_active_after_time))
      && $IsValidU64($SelectField($this, $PackageTxnManager_UpgradePlan_version))
}
function {:inline} $PackageTxnManager_UpgradePlan_$invariant_holds($this: $Value): bool {
    true
}

function {:inline} $PackageTxnManager_UpgradePlan_$is_well_formed($this: $Value): bool {
    $PackageTxnManager_UpgradePlan_$is_well_typed($this) && $PackageTxnManager_UpgradePlan_$invariant_holds($this)}

procedure {:inline 1} $PackageTxnManager_UpgradePlan_pack($file_id: int, $byte_index: int, $var_idx: int, package_hash: $Value, active_after_time: $Value, version: $Value) returns ($struct: $Value)
{
    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));
    assume $IsValidU64(active_after_time);
    assume $IsValidU64(version);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := package_hash][1 := active_after_time][2 := version], 3));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $PackageTxnManager_UpgradePlan_unpack($struct: $Value) returns (package_hash: $Value, active_after_time: $Value, version: $Value)
{
    assume is#$Vector($struct);
    package_hash := $SelectField($struct, $PackageTxnManager_UpgradePlan_package_hash);
    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));
    active_after_time := $SelectField($struct, $PackageTxnManager_UpgradePlan_active_after_time);
    assume $IsValidU64(active_after_time);
    version := $SelectField($struct, $PackageTxnManager_UpgradePlan_version);
    assume $IsValidU64(version);
}

const unique $PackageTxnManager_UpgradePlanCapability: $TypeName;
const $PackageTxnManager_UpgradePlanCapability_account_address: $FieldName;
axiom $PackageTxnManager_UpgradePlanCapability_account_address == 0;
function $PackageTxnManager_UpgradePlanCapability_type_value(): $TypeValue {
    $StructType($PackageTxnManager_UpgradePlanCapability, $EmptyTypeValueArray)
}
var $PackageTxnManager_UpgradePlanCapability_$memory: $Memory;
var $PackageTxnManager_UpgradePlanCapability_$memory_$old: $Memory;
function {:inline} $PackageTxnManager_UpgradePlanCapability_$is_well_typed($this: $Value): bool {
    $Vector_$is_well_formed($this)
    && $vlen($this) == 1
      && is#$Address($SelectField($this, $PackageTxnManager_UpgradePlanCapability_account_address))
}
function {:inline} $PackageTxnManager_UpgradePlanCapability_$invariant_holds($this: $Value): bool {
    true
}

function {:inline} $PackageTxnManager_UpgradePlanCapability_$is_well_formed($this: $Value): bool {
    $PackageTxnManager_UpgradePlanCapability_$is_well_typed($this) && $PackageTxnManager_UpgradePlanCapability_$invariant_holds($this)}

procedure {:inline 1} $PackageTxnManager_UpgradePlanCapability_pack($file_id: int, $byte_index: int, $var_idx: int, account_address: $Value) returns ($struct: $Value)
{
    assume is#$Address(account_address);
    $struct := $Vector($ValueArray($MapConstValue($DefaultValue())[0 := account_address], 1));
    if ($byte_index > 0) {
        if (true) {
         assume {:print "$track_local(",$file_id,",",$byte_index,",",$var_idx,"):", $struct} true;
        }
    }
}

procedure {:inline 1} $PackageTxnManager_UpgradePlanCapability_unpack($struct: $Value) returns (account_address: $Value)
{
    assume is#$Vector($struct);
    account_address := $SelectField($struct, $PackageTxnManager_UpgradePlanCapability_account_address);
    assume is#$Address(account_address);
}



// ** functions of module PackageTxnManager

procedure {:inline 1} $PackageTxnManager_account_address_$def(cap: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t2: $Value; // $AddressType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,5544,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(cap)
    call $t1 := $CopyOrMoveValue(cap);

    // $t2 := get_field<PackageTxnManager::UpgradePlanCapability>.account_address($t1)
    call $t2 := $GetFieldFromValue($t1, $PackageTxnManager_UpgradePlanCapability_account_address);

    // return $t2
    $ret0 := $t2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,5623,3):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $PackageTxnManager_account_address_$direct_inter(cap: $Value) returns ($ret0: $Value)
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $ret0 := $PackageTxnManager_account_address_$def(cap);
}


procedure {:inline 1} $PackageTxnManager_account_address_$direct_intra(cap: $Value) returns ($ret0: $Value)
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $ret0 := $PackageTxnManager_account_address_$def(cap);
}


procedure {:inline 1} $PackageTxnManager_account_address(cap: $Value) returns ($ret0: $Value)
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $ret0 := $PackageTxnManager_account_address_$def(cap);
}


procedure {:inline 1} $PackageTxnManager_account_address_$def_verify(cap: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var $t1: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t2: $Value; // $AddressType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,5544,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(cap)
    call $t1 := $CopyOrMoveValue(cap);

    // $t2 := get_field<PackageTxnManager::UpgradePlanCapability>.account_address($t1)
    call $t2 := $GetFieldFromValue($t1, $PackageTxnManager_UpgradePlanCapability_account_address);

    // return $t2
    $ret0 := $t2;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,5623,3):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:timeLimit 40} $PackageTxnManager_account_address_$verify(cap: $Value) returns ($ret0: $Value)
ensures !$abort_flag;
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $InitVerification();
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_UpgradePlanCapability_$is_well_formed(contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $ret0 := $PackageTxnManager_account_address_$def_verify(cap);
}


procedure {:inline 1} $PackageTxnManager_cancel_upgrade_plan_$def(account: $Value) returns ()
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var cap: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t3: $Value; // $AddressType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,9506,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(account)
    call $t3 := $CopyOrMoveValue(account);

    // account_address := Signer::address_of($t3)
    call account_address := $Signer_address_of($t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,9667):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // cap := get_global<PackageTxnManager::UpgradePlanCapability>(account_address)
    call cap := $GetGlobal($PackageTxnManager_UpgradePlanCapability_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,9710):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,9710,2):", $trace_temp} true;
    }

    // PackageTxnManager::cancel_upgrade_plan_with_cap(cap)
    call $PackageTxnManager_cancel_upgrade_plan_with_cap(cap);
    if ($abort_flag) {
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $PackageTxnManager_cancel_upgrade_plan_$direct_inter(account: $Value) returns ()
{
    assume is#$Address(account);

    call $PackageTxnManager_cancel_upgrade_plan_$def(account);
}


procedure {:inline 1} $PackageTxnManager_cancel_upgrade_plan_$direct_intra(account: $Value) returns ()
{
    assume is#$Address(account);

    call $PackageTxnManager_cancel_upgrade_plan_$def(account);
}


procedure {:inline 1} $PackageTxnManager_cancel_upgrade_plan(account: $Value) returns ()
{
    assume is#$Address(account);

    call $PackageTxnManager_cancel_upgrade_plan_$def(account);
}


procedure {:inline 1} $PackageTxnManager_cancel_upgrade_plan_$def_verify(account: $Value) returns ()
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var cap: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t3: $Value; // $AddressType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,9506,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(account)
    call $t3 := $CopyOrMoveValue(account);

    // account_address := Signer::address_of($t3)
    call account_address := $Signer_address_of_$direct_inter($t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,9667):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // cap := get_global<PackageTxnManager::UpgradePlanCapability>(account_address)
    call cap := $GetGlobal($PackageTxnManager_UpgradePlanCapability_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,9710):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,9710,2):", $trace_temp} true;
    }

    // PackageTxnManager::cancel_upgrade_plan_with_cap(cap)
    call $PackageTxnManager_cancel_upgrade_plan_with_cap_$direct_intra(cap);
    if ($abort_flag) {
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:timeLimit 40} $PackageTxnManager_cancel_upgrade_plan_$verify(account: $Value) returns ()
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!$IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($Option_spec_is_some($PackageTxnManager_UpgradePlan_type_value(), $SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_TwoPhaseUpgrade_plan)))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account))))))
    || b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address))))))
    || b#$Boolean(old($Boolean(!$IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1)))))
    || b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address))))))
    || b#$Boolean(old($Boolean(!b#$Boolean($Option_spec_is_some($PackageTxnManager_UpgradePlan_type_value(), $SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_TwoPhaseUpgrade_plan)))))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_none($PackageTxnManager_UpgradePlan_type_value(), $SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_TwoPhaseUpgrade_plan))));
{
    assume is#$Address(account);

    call $InitVerification();
    assume (forall $inv_addr: int, $inv_tv0: $TypeValue :: {contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr]}
        $Option_Option_$is_well_formed(contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_ModuleUpgradeStrategy_$is_well_formed(contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_TwoPhaseUpgrade_$is_well_formed(contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_UpgradePlanCapability_$is_well_formed(contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $PackageTxnManager_cancel_upgrade_plan_$def_verify(account);
}


procedure {:inline 1} $PackageTxnManager_cancel_upgrade_plan_with_cap_$def(cap: $Value) returns ()
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var tmp#$2: $Value; // $BooleanType()
    var tmp#$3: $Value; // $IntegerType()
    var tmp#$4: $Value; // $BooleanType()
    var tmp#$5: $Value; // $IntegerType()
    var tpu: $Mutation; // ReferenceType($PackageTxnManager_TwoPhaseUpgrade_type_value())
    var $t7: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $BooleanType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t19: $Mutation; // ReferenceType($Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value()))
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,10251,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t7 := move(cap)
    call $t7 := $CopyOrMoveValue(cap);

    // $t8 := get_field<PackageTxnManager::UpgradePlanCapability>.account_address($t7)
    call $t8 := $GetFieldFromValue($t7, $PackageTxnManager_UpgradePlanCapability_account_address);

    // account_address := $t8
    call account_address := $CopyOrMoveValue($t8);
    if (true) {
     $trace_temp := account_address;
      assume {:print "$track_local(7,10384,1):", $trace_temp} true;
    }

    // $t9 := PackageTxnManager::get_module_upgrade_strategy(account_address)
    call $t9 := $PackageTxnManager_get_module_upgrade_strategy(account_address);
    if ($abort_flag) {
      goto Abort;
    }

    // $t10 := 1
    $t10 := $Integer(1);

    // $t11 := ==($t9, $t10)
    $t11 := $Boolean($IsEqual($t9, $t10));

    // $t12 := 107
    $t12 := $Integer(107);

    // $t13 := Errors::invalid_argument($t12)
    call $t13 := $Errors_invalid_argument($t12);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10518):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t11) goto L0 else goto L1
    if (b#$Boolean($t11)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // abort($t13)
    $abort_code := i#$Integer($t13);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10435):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // tpu := borrow_global<PackageTxnManager::TwoPhaseUpgrade>(account_address)
    call tpu := $BorrowGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10584):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$unpack_ref($Dereference(tpu));

    // $t14 := get_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t14 := $GetFieldFromReference(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);
    assert $Option_Option_$invariant_holds($t14);

    // $t15 := Option::is_some<PackageTxnManager::UpgradePlan>($t14)
    call $t15 := $Option_is_some($PackageTxnManager_UpgradePlan_type_value(), $t14);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10664):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t16 := 102
    $t16 := $Integer(102);

    // $t17 := Errors::invalid_state($t16)
    call $t17 := $Errors_invalid_state($t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10692):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t15) goto L2 else goto L3
    if (b#$Boolean($t15)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // destroy(tpu)

    // pack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$pack_ref($Dereference(tpu));

    // abort($t17)
    $abort_code := i#$Integer($t17);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10649):", $trace_abort_temp} true;
    }
    goto Abort;

    // L2:
L2:

    // $t18 := Option::none<PackageTxnManager::UpgradePlan>()
    call $t18 := $Option_none($PackageTxnManager_UpgradePlan_type_value());
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10762):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t19 := borrow_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t19 := $BorrowField(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);

    // unpack_ref($t19)
    call $Option_Option_$unpack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t19));

    // write_ref($t19, $t18)
    call $t19 := $WriteRef($t19, $t18);
    if (true) {
     $trace_temp := $Dereference(tpu);
      assume {:print "$track_local(7,10743,6):", $trace_temp} true;
    }

    // pack_ref($t19)
    call $Option_Option_$pack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t19));

    // write_back[Reference(tpu)]($t19)
    call tpu := $WritebackToReference($t19, tpu);

    // pack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$pack_ref($Dereference(tpu));

    // write_back[PackageTxnManager::TwoPhaseUpgrade](tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$memory := $WritebackToGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, tpu);

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $PackageTxnManager_cancel_upgrade_plan_with_cap_$direct_inter(cap: $Value) returns ()
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $PackageTxnManager_cancel_upgrade_plan_with_cap_$def(cap);
}


procedure {:inline 1} $PackageTxnManager_cancel_upgrade_plan_with_cap_$direct_intra(cap: $Value) returns ()
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $PackageTxnManager_cancel_upgrade_plan_with_cap_$def(cap);
}


procedure {:inline 1} $PackageTxnManager_cancel_upgrade_plan_with_cap(cap: $Value) returns ()
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $PackageTxnManager_cancel_upgrade_plan_with_cap_$def(cap);
}


procedure {:inline 1} $PackageTxnManager_cancel_upgrade_plan_with_cap_$def_verify(cap: $Value) returns ()
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var tmp#$2: $Value; // $BooleanType()
    var tmp#$3: $Value; // $IntegerType()
    var tmp#$4: $Value; // $BooleanType()
    var tmp#$5: $Value; // $IntegerType()
    var tpu: $Mutation; // ReferenceType($PackageTxnManager_TwoPhaseUpgrade_type_value())
    var $t7: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t8: $Value; // $AddressType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $IntegerType()
    var $t11: $Value; // $BooleanType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t19: $Mutation; // ReferenceType($Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value()))
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,10251,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t7 := move(cap)
    call $t7 := $CopyOrMoveValue(cap);

    // $t8 := get_field<PackageTxnManager::UpgradePlanCapability>.account_address($t7)
    call $t8 := $GetFieldFromValue($t7, $PackageTxnManager_UpgradePlanCapability_account_address);

    // account_address := $t8
    call account_address := $CopyOrMoveValue($t8);
    if (true) {
     $trace_temp := account_address;
      assume {:print "$track_local(7,10384,1):", $trace_temp} true;
    }

    // $t9 := PackageTxnManager::get_module_upgrade_strategy(account_address)
    call $t9 := $PackageTxnManager_get_module_upgrade_strategy_$direct_intra(account_address);
    if ($abort_flag) {
      goto Abort;
    }

    // $t10 := 1
    $t10 := $Integer(1);

    // $t11 := ==($t9, $t10)
    $t11 := $Boolean($IsEqual($t9, $t10));

    // $t12 := 107
    $t12 := $Integer(107);

    // $t13 := Errors::invalid_argument($t12)
    call $t13 := $Errors_invalid_argument_$direct_inter($t12);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10518):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t11) goto L0 else goto L1
    if (b#$Boolean($t11)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // abort($t13)
    $abort_code := i#$Integer($t13);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10435):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // tpu := borrow_global<PackageTxnManager::TwoPhaseUpgrade>(account_address)
    call tpu := $BorrowGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10584):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$unpack_ref($Dereference(tpu));

    // $t14 := get_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t14 := $GetFieldFromReference(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);
    assert $Option_Option_$invariant_holds($t14);

    // $t15 := Option::is_some<PackageTxnManager::UpgradePlan>($t14)
    call $t15 := $Option_is_some_$direct_inter($PackageTxnManager_UpgradePlan_type_value(), $t14);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10664):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t16 := 102
    $t16 := $Integer(102);

    // $t17 := Errors::invalid_state($t16)
    call $t17 := $Errors_invalid_state_$direct_inter($t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10692):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t15) goto L2 else goto L3
    if (b#$Boolean($t15)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // destroy(tpu)

    // pack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$pack_ref($Dereference(tpu));

    // abort($t17)
    $abort_code := i#$Integer($t17);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10649):", $trace_abort_temp} true;
    }
    goto Abort;

    // L2:
L2:

    // $t18 := Option::none<PackageTxnManager::UpgradePlan>()
    call $t18 := $Option_none_$direct_inter($PackageTxnManager_UpgradePlan_type_value());
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,10762):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t19 := borrow_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t19 := $BorrowField(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);

    // unpack_ref($t19)
    call $Option_Option_$unpack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t19));

    // write_ref($t19, $t18)
    call $t19 := $WriteRef($t19, $t18);
    if (true) {
     $trace_temp := $Dereference(tpu);
      assume {:print "$track_local(7,10743,6):", $trace_temp} true;
    }

    // pack_ref($t19)
    call $Option_Option_$pack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t19));

    // write_back[Reference(tpu)]($t19)
    call tpu := $WritebackToReference($t19, tpu);

    // pack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$pack_ref($Dereference(tpu));

    // write_back[PackageTxnManager::TwoPhaseUpgrade](tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$memory := $WritebackToGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, tpu);

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:timeLimit 40} $PackageTxnManager_cancel_upgrade_plan_with_cap_$verify(cap: $Value) returns ()
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!$IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($Option_spec_is_some($PackageTxnManager_UpgradePlan_type_value(), $SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_TwoPhaseUpgrade_plan)))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address))))))
    || b#$Boolean(old($Boolean(!$IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1)))))
    || b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address))))))
    || b#$Boolean(old($Boolean(!b#$Boolean($Option_spec_is_some($PackageTxnManager_UpgradePlan_type_value(), $SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_TwoPhaseUpgrade_plan)))))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_none($PackageTxnManager_UpgradePlan_type_value(), $SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_TwoPhaseUpgrade_plan))));
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $InitVerification();
    assume (forall $inv_addr: int, $inv_tv0: $TypeValue :: {contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr]}
        $Option_Option_$is_well_formed(contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_ModuleUpgradeStrategy_$is_well_formed(contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_TwoPhaseUpgrade_$is_well_formed(contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_UpgradePlanCapability_$is_well_formed(contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $PackageTxnManager_cancel_upgrade_plan_with_cap_$def_verify(cap);
}


procedure {:inline 1} $PackageTxnManager_check_package_txn_$def(package_address: $Value, package_hash: $Value) returns ()
{
    // declare local variables
    var plan: $Value; // $PackageTxnManager_UpgradePlan_type_value()
    var plan_opt: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var strategy: $Value; // $IntegerType()
    var tmp#$5: $Value; // $BooleanType()
    var tmp#$6: $Value; // $IntegerType()
    var tmp#$7: $Value; // $BooleanType()
    var tmp#$8: $Value; // $IntegerType()
    var tmp#$9: $Value; // $BooleanType()
    var tmp#$10: $Value; // $IntegerType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $Vector_type_value($IntegerType())
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $BooleanType()
    var $t17: $Value; // $BooleanType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $Vector_type_value($IntegerType())
    var $t21: $Value; // $BooleanType()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $IntegerType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $BooleanType()
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $IntegerType()
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $BooleanType()
    var $t31: $Value; // $IntegerType()
    var $t32: $Value; // $BooleanType()
    var $t33: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := package_address;
      assume {:print "$track_local(7,12802,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_hash;
      assume {:print "$track_local(7,12802,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t11 := move(package_address)
    call $t11 := $CopyOrMoveValue(package_address);

    // $t12 := move(package_hash)
    call $t12 := $CopyOrMoveValue(package_hash);

    // strategy := PackageTxnManager::get_module_upgrade_strategy($t11)
    call strategy := $PackageTxnManager_get_module_upgrade_strategy($t11);
    if ($abort_flag) {
      goto Abort;
    }

    // $t13 := 0
    $t13 := $Integer(0);

    // $t14 := ==(strategy, $t13)
    $t14 := $Boolean($IsEqual(strategy, $t13));

    // if ($t14) goto L0 else goto L1
    if (b#$Boolean($t14)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // goto L3
    goto L3;

    // L2:
L2:

    // $t15 := 1
    $t15 := $Integer(1);

    // $t16 := ==(strategy, $t15)
    $t16 := $Boolean($IsEqual(strategy, $t15));

    // if ($t16) goto L4 else goto L5
    if (b#$Boolean($t16)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // goto L6
    goto L6;

    // L4:
L4:

    // plan_opt := PackageTxnManager::get_upgrade_plan($t11)
    call plan_opt := $PackageTxnManager_get_upgrade_plan($t11);
    if ($abort_flag) {
      goto Abort;
    }

    // $t17 := Option::is_some<PackageTxnManager::UpgradePlan>(plan_opt)
    call $t17 := $Option_is_some($PackageTxnManager_UpgradePlan_type_value(), plan_opt);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13234):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t18 := 102
    $t18 := $Integer(102);

    // $t19 := Errors::invalid_argument($t18)
    call $t19 := $Errors_invalid_argument($t18);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13262):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t17) goto L7 else goto L8
    if (b#$Boolean($t17)) { goto L7; } else { goto L8; }

    // L8:
L8:

    // abort($t19)
    $abort_code := i#$Integer($t19);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13219):", $trace_abort_temp} true;
    }
    goto Abort;

    // L7:
L7:

    // plan := Option::borrow<PackageTxnManager::UpgradePlan>(plan_opt)
    call plan := $Option_borrow($PackageTxnManager_UpgradePlan_type_value(), plan_opt);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13339):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t20 := get_field<PackageTxnManager::UpgradePlan>.package_hash(plan)
    call $t20 := $GetFieldFromValue(plan, $PackageTxnManager_UpgradePlan_package_hash);

    // $t21 := ==($t20, $t12)
    $t21 := $Boolean($IsEqual($t20, $t12));

    // $t22 := 103
    $t22 := $Integer(103);

    // $t23 := Errors::invalid_argument($t22)
    call $t23 := $Errors_invalid_argument($t22);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13426):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t21) goto L9 else goto L10
    if (b#$Boolean($t21)) { goto L9; } else { goto L10; }

    // L10:
L10:

    // destroy(plan)

    // abort($t23)
    $abort_code := i#$Integer($t23);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13374):", $trace_abort_temp} true;
    }
    goto Abort;

    // L9:
L9:

    // $t24 := get_field<PackageTxnManager::UpgradePlan>.active_after_time(plan)
    call $t24 := $GetFieldFromValue(plan, $PackageTxnManager_UpgradePlan_active_after_time);

    // $t25 := Timestamp::now_milliseconds()
    call $t25 := $Timestamp_now_milliseconds();
    if ($abort_flag) {
      goto Abort;
    }

    // $t26 := <=($t24, $t25)
    call $t26 := $Le($t24, $t25);

    // $t27 := 104
    $t27 := $Integer(104);

    // $t28 := Errors::invalid_argument($t27)
    call $t28 := $Errors_invalid_argument($t27);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13558):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t26) goto L11 else goto L12
    if (b#$Boolean($t26)) { goto L11; } else { goto L12; }

    // L12:
L12:

    // abort($t28)
    $abort_code := i#$Integer($t28);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13486):", $trace_abort_temp} true;
    }
    goto Abort;

    // L11:
L11:

    // goto L3
    goto L3;

    // L6:
L6:

    // $t29 := 2
    $t29 := $Integer(2);

    // $t30 := ==(strategy, $t29)
    $t30 := $Boolean($IsEqual(strategy, $t29));

    // if ($t30) goto L13 else goto L14
    if (b#$Boolean($t30)) { goto L13; } else { goto L14; }

    // L14:
L14:

    // goto L15
    goto L15;

    // L13:
L13:

    // goto L3
    goto L3;

    // L15:
L15:

    // $t31 := 3
    $t31 := $Integer(3);

    // $t32 := ==(strategy, $t31)
    $t32 := $Boolean($IsEqual(strategy, $t31));

    // if ($t32) goto L16 else goto L17
    if (b#$Boolean($t32)) { goto L16; } else { goto L17; }

    // L17:
L17:

    // goto L3
    goto L3;

    // L16:
L16:

    // $t33 := 105
    $t33 := $Integer(105);

    // abort($t33)
    $abort_code := i#$Integer($t33);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13765):", $trace_abort_temp} true;
    }
    goto Abort;

    // L3:
L3:

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $PackageTxnManager_check_package_txn_$direct_inter(package_address: $Value, package_hash: $Value) returns ()
{
    assume is#$Address(package_address);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    call $PackageTxnManager_check_package_txn_$def(package_address, package_hash);
}


procedure {:inline 1} $PackageTxnManager_check_package_txn_$direct_intra(package_address: $Value, package_hash: $Value) returns ()
{
    assume is#$Address(package_address);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    call $PackageTxnManager_check_package_txn_$def(package_address, package_hash);
}


procedure {:inline 1} $PackageTxnManager_check_package_txn(package_address: $Value, package_hash: $Value) returns ()
{
    assume is#$Address(package_address);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    call $PackageTxnManager_check_package_txn_$def(package_address, package_hash);
}


procedure {:inline 1} $PackageTxnManager_check_package_txn_$def_verify(package_address: $Value, package_hash: $Value) returns ()
{
    // declare local variables
    var plan: $Value; // $PackageTxnManager_UpgradePlan_type_value()
    var plan_opt: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var strategy: $Value; // $IntegerType()
    var tmp#$5: $Value; // $BooleanType()
    var tmp#$6: $Value; // $IntegerType()
    var tmp#$7: $Value; // $BooleanType()
    var tmp#$8: $Value; // $IntegerType()
    var tmp#$9: $Value; // $BooleanType()
    var tmp#$10: $Value; // $IntegerType()
    var $t11: $Value; // $AddressType()
    var $t12: $Value; // $Vector_type_value($IntegerType())
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $BooleanType()
    var $t17: $Value; // $BooleanType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $Vector_type_value($IntegerType())
    var $t21: $Value; // $BooleanType()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $IntegerType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $BooleanType()
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $IntegerType()
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $BooleanType()
    var $t31: $Value; // $IntegerType()
    var $t32: $Value; // $BooleanType()
    var $t33: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := package_address;
      assume {:print "$track_local(7,12802,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_hash;
      assume {:print "$track_local(7,12802,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t11 := move(package_address)
    call $t11 := $CopyOrMoveValue(package_address);

    // $t12 := move(package_hash)
    call $t12 := $CopyOrMoveValue(package_hash);

    // strategy := PackageTxnManager::get_module_upgrade_strategy($t11)
    call strategy := $PackageTxnManager_get_module_upgrade_strategy_$direct_intra($t11);
    if ($abort_flag) {
      goto Abort;
    }

    // $t13 := 0
    $t13 := $Integer(0);

    // $t14 := ==(strategy, $t13)
    $t14 := $Boolean($IsEqual(strategy, $t13));

    // if ($t14) goto L0 else goto L1
    if (b#$Boolean($t14)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // goto L3
    goto L3;

    // L2:
L2:

    // $t15 := 1
    $t15 := $Integer(1);

    // $t16 := ==(strategy, $t15)
    $t16 := $Boolean($IsEqual(strategy, $t15));

    // if ($t16) goto L4 else goto L5
    if (b#$Boolean($t16)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // goto L6
    goto L6;

    // L4:
L4:

    // plan_opt := PackageTxnManager::get_upgrade_plan($t11)
    call plan_opt := $PackageTxnManager_get_upgrade_plan_$direct_intra($t11);
    if ($abort_flag) {
      goto Abort;
    }

    // $t17 := Option::is_some<PackageTxnManager::UpgradePlan>(plan_opt)
    call $t17 := $Option_is_some_$direct_inter($PackageTxnManager_UpgradePlan_type_value(), plan_opt);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13234):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t18 := 102
    $t18 := $Integer(102);

    // $t19 := Errors::invalid_argument($t18)
    call $t19 := $Errors_invalid_argument_$direct_inter($t18);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13262):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t17) goto L7 else goto L8
    if (b#$Boolean($t17)) { goto L7; } else { goto L8; }

    // L8:
L8:

    // abort($t19)
    $abort_code := i#$Integer($t19);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13219):", $trace_abort_temp} true;
    }
    goto Abort;

    // L7:
L7:

    // plan := Option::borrow<PackageTxnManager::UpgradePlan>(plan_opt)
    call plan := $Option_borrow_$direct_inter($PackageTxnManager_UpgradePlan_type_value(), plan_opt);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13339):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t20 := get_field<PackageTxnManager::UpgradePlan>.package_hash(plan)
    call $t20 := $GetFieldFromValue(plan, $PackageTxnManager_UpgradePlan_package_hash);

    // $t21 := ==($t20, $t12)
    $t21 := $Boolean($IsEqual($t20, $t12));

    // $t22 := 103
    $t22 := $Integer(103);

    // $t23 := Errors::invalid_argument($t22)
    call $t23 := $Errors_invalid_argument_$direct_inter($t22);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13426):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t21) goto L9 else goto L10
    if (b#$Boolean($t21)) { goto L9; } else { goto L10; }

    // L10:
L10:

    // destroy(plan)

    // abort($t23)
    $abort_code := i#$Integer($t23);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13374):", $trace_abort_temp} true;
    }
    goto Abort;

    // L9:
L9:

    // $t24 := get_field<PackageTxnManager::UpgradePlan>.active_after_time(plan)
    call $t24 := $GetFieldFromValue(plan, $PackageTxnManager_UpgradePlan_active_after_time);

    // $t25 := Timestamp::now_milliseconds()
    call $t25 := $Timestamp_now_milliseconds_$direct_inter();
    if ($abort_flag) {
      goto Abort;
    }

    // $t26 := <=($t24, $t25)
    call $t26 := $Le($t24, $t25);

    // $t27 := 104
    $t27 := $Integer(104);

    // $t28 := Errors::invalid_argument($t27)
    call $t28 := $Errors_invalid_argument_$direct_inter($t27);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13558):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t26) goto L11 else goto L12
    if (b#$Boolean($t26)) { goto L11; } else { goto L12; }

    // L12:
L12:

    // abort($t28)
    $abort_code := i#$Integer($t28);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13486):", $trace_abort_temp} true;
    }
    goto Abort;

    // L11:
L11:

    // goto L3
    goto L3;

    // L6:
L6:

    // $t29 := 2
    $t29 := $Integer(2);

    // $t30 := ==(strategy, $t29)
    $t30 := $Boolean($IsEqual(strategy, $t29));

    // if ($t30) goto L13 else goto L14
    if (b#$Boolean($t30)) { goto L13; } else { goto L14; }

    // L14:
L14:

    // goto L15
    goto L15;

    // L13:
L13:

    // goto L3
    goto L3;

    // L15:
L15:

    // $t31 := 3
    $t31 := $Integer(3);

    // $t32 := ==(strategy, $t31)
    $t32 := $Boolean($IsEqual(strategy, $t31));

    // if ($t32) goto L16 else goto L17
    if (b#$Boolean($t32)) { goto L16; } else { goto L17; }

    // L17:
L17:

    // goto L3
    goto L3;

    // L16:
L16:

    // $t33 := 105
    $t33 := $Integer(105);

    // abort($t33)
    $abort_code := i#$Integer($t33);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,13765):", $trace_abort_temp} true;
    }
    goto Abort;

    // L3:
L3:

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:timeLimit 40} $PackageTxnManager_check_package_txn_$verify(package_address: $Value, package_hash: $Value) returns ()
ensures b#$Boolean(old($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(3))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Option_spec_is_none($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(!$IsEqual($SelectField($Option_spec_get($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address)), $PackageTxnManager_UpgradePlan_package_hash), package_hash)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS()))))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(i#$Integer($SelectField($Option_spec_get($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address)), $PackageTxnManager_UpgradePlan_active_after_time)) > i#$Integer($Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory))))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(3)))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Option_spec_is_none($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(!$IsEqual($SelectField($Option_spec_get($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address)), $PackageTxnManager_UpgradePlan_package_hash), package_hash))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS())))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(i#$Integer($SelectField($Option_spec_get($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address)), $PackageTxnManager_UpgradePlan_active_after_time)) > i#$Integer($Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory))))))));
{
    assume is#$Address(package_address);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    call $InitVerification();
    assume (forall $inv_addr: int, $inv_tv0: $TypeValue :: {contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr]}
        $Option_Option_$is_well_formed(contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($Timestamp_CurrentTimeMilliseconds_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $Timestamp_CurrentTimeMilliseconds_$is_well_formed(contents#$Memory($Timestamp_CurrentTimeMilliseconds_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_ModuleUpgradeStrategy_$is_well_formed(contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_TwoPhaseUpgrade_$is_well_formed(contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_UpgradePlan_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_UpgradePlan_$is_well_formed(contents#$Memory($PackageTxnManager_UpgradePlan_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $PackageTxnManager_check_package_txn_$def_verify(package_address, package_hash);
}


procedure {:inline 1} $PackageTxnManager_destroy_upgrade_plan_cap_$def(cap: $Value) returns ()
{
    // declare local variables
    var $t1: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t2: $Value; // $AddressType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,5662,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(cap)
    call $t1 := $CopyOrMoveValue(cap);

    // $t2 := unpack PackageTxnManager::UpgradePlanCapability($t1)
    call $t2 := $PackageTxnManager_UpgradePlanCapability_unpack($t1);


    // destroy($t2)

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $PackageTxnManager_destroy_upgrade_plan_cap_$direct_inter(cap: $Value) returns ()
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $PackageTxnManager_destroy_upgrade_plan_cap_$def(cap);
}


procedure {:inline 1} $PackageTxnManager_destroy_upgrade_plan_cap_$direct_intra(cap: $Value) returns ()
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $PackageTxnManager_destroy_upgrade_plan_cap_$def(cap);
}


procedure {:inline 1} $PackageTxnManager_destroy_upgrade_plan_cap(cap: $Value) returns ()
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $PackageTxnManager_destroy_upgrade_plan_cap_$def(cap);
}


procedure {:inline 1} $PackageTxnManager_destroy_upgrade_plan_cap_$def_verify(cap: $Value) returns ()
{
    // declare local variables
    var $t1: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t2: $Value; // $AddressType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,5662,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t1 := move(cap)
    call $t1 := $CopyOrMoveValue(cap);

    // $t2 := unpack PackageTxnManager::UpgradePlanCapability($t1)
    call $t2 := $PackageTxnManager_UpgradePlanCapability_unpack($t1);


    // destroy($t2)

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:timeLimit 40} $PackageTxnManager_destroy_upgrade_plan_cap_$verify(cap: $Value) returns ()
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    call $InitVerification();
    call $PackageTxnManager_destroy_upgrade_plan_cap_$def_verify(cap);
}


procedure {:inline 1} $PackageTxnManager_extract_submit_upgrade_plan_cap_$def(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var tmp#$2: $Value; // $BooleanType()
    var tmp#$3: $Value; // $IntegerType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,5894,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t4 := move(account)
    call $t4 := $CopyOrMoveValue(account);

    // account_address := Signer::address_of($t4)
    call account_address := $Signer_address_of($t4);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6075):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t5 := PackageTxnManager::get_module_upgrade_strategy(account_address)
    call $t5 := $PackageTxnManager_get_module_upgrade_strategy(account_address);
    if ($abort_flag) {
      goto Abort;
    }

    // $t6 := 1
    $t6 := $Integer(1);

    // $t7 := ==($t5, $t6)
    $t7 := $Boolean($IsEqual($t5, $t6));

    // $t8 := 107
    $t8 := $Integer(107);

    // $t9 := Errors::invalid_argument($t8)
    call $t9 := $Errors_invalid_argument($t8);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6191):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t7) goto L0 else goto L1
    if (b#$Boolean($t7)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // abort($t9)
    $abort_code := i#$Integer($t9);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6108):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // $t10 := move_from<PackageTxnManager::UpgradePlanCapability>(account_address)
    call $PackageTxnManager_UpgradePlanCapability_$memory, $t10 := $MoveFrom($PackageTxnManager_UpgradePlanCapability_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6247):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t10
    $ret0 := $t10;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,6247,11):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $PackageTxnManager_extract_submit_upgrade_plan_cap_$direct_inter(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $PackageTxnManager_extract_submit_upgrade_plan_cap_$def(account);
}


procedure {:inline 1} $PackageTxnManager_extract_submit_upgrade_plan_cap_$direct_intra(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $PackageTxnManager_extract_submit_upgrade_plan_cap_$def(account);
}


procedure {:inline 1} $PackageTxnManager_extract_submit_upgrade_plan_cap(account: $Value) returns ($ret0: $Value)
{
    assume is#$Address(account);

    call $ret0 := $PackageTxnManager_extract_submit_upgrade_plan_cap_$def(account);
}


procedure {:inline 1} $PackageTxnManager_extract_submit_upgrade_plan_cap_$def_verify(account: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var tmp#$2: $Value; // $BooleanType()
    var tmp#$3: $Value; // $IntegerType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $IntegerType()
    var $t10: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,5894,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t4 := move(account)
    call $t4 := $CopyOrMoveValue(account);

    // account_address := Signer::address_of($t4)
    call account_address := $Signer_address_of_$direct_inter($t4);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6075):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t5 := PackageTxnManager::get_module_upgrade_strategy(account_address)
    call $t5 := $PackageTxnManager_get_module_upgrade_strategy_$direct_intra(account_address);
    if ($abort_flag) {
      goto Abort;
    }

    // $t6 := 1
    $t6 := $Integer(1);

    // $t7 := ==($t5, $t6)
    $t7 := $Boolean($IsEqual($t5, $t6));

    // $t8 := 107
    $t8 := $Integer(107);

    // $t9 := Errors::invalid_argument($t8)
    call $t9 := $Errors_invalid_argument_$direct_inter($t8);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6191):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t7) goto L0 else goto L1
    if (b#$Boolean($t7)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // abort($t9)
    $abort_code := i#$Integer($t9);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6108):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // $t10 := move_from<PackageTxnManager::UpgradePlanCapability>(account_address)
    call $PackageTxnManager_UpgradePlanCapability_$memory, $t10 := $MoveFrom($PackageTxnManager_UpgradePlanCapability_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6247):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // return $t10
    $ret0 := $t10;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,6247,11):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:timeLimit 40} $PackageTxnManager_extract_submit_upgrade_plan_cap_$verify(account: $Value) returns ($ret0: $Value)
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!$IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account))))))
    || b#$Boolean(old($Boolean(!$IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1)))))
    || b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)))))));
{
    assume is#$Address(account);

    call $InitVerification();
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_ModuleUpgradeStrategy_$is_well_formed(contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_UpgradePlanCapability_$is_well_formed(contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $ret0 := $PackageTxnManager_extract_submit_upgrade_plan_cap_$def_verify(account);
}


procedure {:inline 1} $PackageTxnManager_finish_upgrade_plan_$def(package_address: $Value) returns ()
{
    // declare local variables
    var plan: $Value; // $PackageTxnManager_UpgradePlan_type_value()
    var tpu: $Mutation; // ReferenceType($PackageTxnManager_TwoPhaseUpgrade_type_value())
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t7: $Mutation; // ReferenceType($Config_ModifyConfigCapability_type_value($Version_Version_type_value()))
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $Version_Version_type_value()
    var $t10: $Value; // $Config_ModifyConfigCapability_type_value($Version_Version_type_value())
    var $t11: $Mutation; // ReferenceType($Event_EventHandle_type_value($PackageTxnManager_UpgradeEvent_type_value()))
    var $t12: $Value; // $Vector_type_value($IntegerType())
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $PackageTxnManager_UpgradeEvent_type_value()
    var $t15: $Value; // $Event_EventHandle_type_value($PackageTxnManager_UpgradeEvent_type_value())
    var $t16: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t17: $Mutation; // ReferenceType($Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value()))
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := package_address;
      assume {:print "$track_local(7,15788,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(package_address)
    call $t3 := $CopyOrMoveValue(package_address);

    // tpu := borrow_global<PackageTxnManager::TwoPhaseUpgrade>($t3)
    call tpu := $BorrowGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, $t3, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,15887):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$unpack_ref($Dereference(tpu));

    // $t4 := get_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t4 := $GetFieldFromReference(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);
    assert $Option_Option_$invariant_holds($t4);

    // $t5 := Option::is_some<PackageTxnManager::UpgradePlan>($t4)
    call $t5 := $Option_is_some($PackageTxnManager_UpgradePlan_type_value(), $t4);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,15964):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t5) goto L0 else goto L1
    if (b#$Boolean($t5)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t6 := get_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t6 := $GetFieldFromReference(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);
    assert $Option_Option_$invariant_holds($t6);

    // plan := Option::borrow<PackageTxnManager::UpgradePlan>($t6)
    call plan := $Option_borrow($PackageTxnManager_UpgradePlan_type_value(), $t6);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,16021):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t7 := borrow_field<PackageTxnManager::TwoPhaseUpgrade>.version_cap(tpu)
    call $t7 := $BorrowField(tpu, $PackageTxnManager_TwoPhaseUpgrade_version_cap);

    // unpack_ref($t7)

    // $t8 := get_field<PackageTxnManager::UpgradePlan>.version(plan)
    call $t8 := $GetFieldFromValue(plan, $PackageTxnManager_UpgradePlan_version);

    // $t9 := Version::new_version($t8)
    call $t9 := $Version_new_version($t8);
    if ($abort_flag) {
      goto Abort;
    }

    // $t10 := read_ref($t7)
    call $t10 := $ReadRef($t7);
    assert $Config_ModifyConfigCapability_$invariant_holds($t10);

    // $t10 := Config::set_with_capability<Version::Version>($t10, $t9)
    call $t10 := $Config_set_with_capability($Version_Version_type_value(), $t10, $t9);
    if ($abort_flag) {
      goto Abort;
    }

    // write_ref($t7, $t10)
    call $t7 := $WriteRef($t7, $t10);
    if (true) {
     $trace_temp := $Dereference(tpu);
      assume {:print "$track_local(7,16064,2):", $trace_temp} true;
    }

    // pack_ref($t7)

    // write_back[Reference(tpu)]($t7)
    call tpu := $WritebackToReference($t7, tpu);

    // $t11 := borrow_field<PackageTxnManager::TwoPhaseUpgrade>.upgrade_event(tpu)
    call $t11 := $BorrowField(tpu, $PackageTxnManager_TwoPhaseUpgrade_upgrade_event);

    // unpack_ref($t11)

    // $t12 := get_field<PackageTxnManager::UpgradePlan>.package_hash(plan)
    call $t12 := $GetFieldFromValue(plan, $PackageTxnManager_UpgradePlan_package_hash);

    // $t13 := get_field<PackageTxnManager::UpgradePlan>.version(plan)
    call $t13 := $GetFieldFromValue(plan, $PackageTxnManager_UpgradePlan_version);

    // $t14 := pack PackageTxnManager::UpgradeEvent($t3, $t12, $t13)
    call $t14 := $PackageTxnManager_UpgradeEvent_pack(0, 0, 0, $t3, $t12, $t13);

    // $t15 := read_ref($t11)
    call $t15 := $ReadRef($t11);
    assert $Event_EventHandle_$invariant_holds($t15);

    // $t15 := Event::emit_event<PackageTxnManager::UpgradeEvent>($t15, $t14)
    call $t15 := $Event_emit_event($PackageTxnManager_UpgradeEvent_type_value(), $t15, $t14);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,16184):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t11, $t15)
    call $t11 := $WriteRef($t11, $t15);
    if (true) {
     $trace_temp := $Dereference(tpu);
      assume {:print "$track_local(7,16184,2):", $trace_temp} true;
    }

    // pack_ref($t11)

    // write_back[Reference(tpu)]($t11)
    call tpu := $WritebackToReference($t11, tpu);

    // goto L2
    goto L2;

    // L2:
L2:

    // $t16 := Option::none<PackageTxnManager::UpgradePlan>()
    call $t16 := $Option_none($PackageTxnManager_UpgradePlan_type_value());
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,16454):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t17 := borrow_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t17 := $BorrowField(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);

    // unpack_ref($t17)
    call $Option_Option_$unpack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t17));

    // write_ref($t17, $t16)
    call $t17 := $WriteRef($t17, $t16);
    if (true) {
     $trace_temp := $Dereference(tpu);
      assume {:print "$track_local(7,16435,2):", $trace_temp} true;
    }

    // pack_ref($t17)
    call $Option_Option_$pack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t17));

    // write_back[Reference(tpu)]($t17)
    call tpu := $WritebackToReference($t17, tpu);

    // pack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$pack_ref($Dereference(tpu));

    // write_back[PackageTxnManager::TwoPhaseUpgrade](tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$memory := $WritebackToGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, tpu);

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $PackageTxnManager_finish_upgrade_plan_$direct_intra(package_address: $Value) returns ()
{
    assume is#$Address(package_address);

    call $PackageTxnManager_finish_upgrade_plan_$def(package_address);
}


procedure {:inline 1} $PackageTxnManager_finish_upgrade_plan(package_address: $Value) returns ()
{
    assume is#$Address(package_address);

    call $PackageTxnManager_finish_upgrade_plan_$def(package_address);
}


procedure {:inline 1} $PackageTxnManager_finish_upgrade_plan_$def_verify(package_address: $Value) returns ()
{
    // declare local variables
    var plan: $Value; // $PackageTxnManager_UpgradePlan_type_value()
    var tpu: $Mutation; // ReferenceType($PackageTxnManager_TwoPhaseUpgrade_type_value())
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t5: $Value; // $BooleanType()
    var $t6: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t7: $Mutation; // ReferenceType($Config_ModifyConfigCapability_type_value($Version_Version_type_value()))
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $Version_Version_type_value()
    var $t10: $Value; // $Config_ModifyConfigCapability_type_value($Version_Version_type_value())
    var $t11: $Mutation; // ReferenceType($Event_EventHandle_type_value($PackageTxnManager_UpgradeEvent_type_value()))
    var $t12: $Value; // $Vector_type_value($IntegerType())
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $PackageTxnManager_UpgradeEvent_type_value()
    var $t15: $Value; // $Event_EventHandle_type_value($PackageTxnManager_UpgradeEvent_type_value())
    var $t16: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t17: $Mutation; // ReferenceType($Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value()))
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := package_address;
      assume {:print "$track_local(7,15788,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(package_address)
    call $t3 := $CopyOrMoveValue(package_address);

    // tpu := borrow_global<PackageTxnManager::TwoPhaseUpgrade>($t3)
    call tpu := $BorrowGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, $t3, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,15887):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$unpack_ref($Dereference(tpu));

    // $t4 := get_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t4 := $GetFieldFromReference(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);
    assert $Option_Option_$invariant_holds($t4);

    // $t5 := Option::is_some<PackageTxnManager::UpgradePlan>($t4)
    call $t5 := $Option_is_some_$direct_inter($PackageTxnManager_UpgradePlan_type_value(), $t4);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,15964):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t5) goto L0 else goto L1
    if (b#$Boolean($t5)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t6 := get_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t6 := $GetFieldFromReference(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);
    assert $Option_Option_$invariant_holds($t6);

    // plan := Option::borrow<PackageTxnManager::UpgradePlan>($t6)
    call plan := $Option_borrow_$direct_inter($PackageTxnManager_UpgradePlan_type_value(), $t6);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,16021):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t7 := borrow_field<PackageTxnManager::TwoPhaseUpgrade>.version_cap(tpu)
    call $t7 := $BorrowField(tpu, $PackageTxnManager_TwoPhaseUpgrade_version_cap);

    // unpack_ref($t7)

    // $t8 := get_field<PackageTxnManager::UpgradePlan>.version(plan)
    call $t8 := $GetFieldFromValue(plan, $PackageTxnManager_UpgradePlan_version);

    // $t9 := Version::new_version($t8)
    call $t9 := $Version_new_version_$direct_inter($t8);
    if ($abort_flag) {
      goto Abort;
    }

    // $t10 := read_ref($t7)
    call $t10 := $ReadRef($t7);
    assert $Config_ModifyConfigCapability_$invariant_holds($t10);

    // $t10 := Config::set_with_capability<Version::Version>($t10, $t9)
    call $t10 := $Config_set_with_capability_$direct_inter($Version_Version_type_value(), $t10, $t9);
    if ($abort_flag) {
      goto Abort;
    }

    // write_ref($t7, $t10)
    call $t7 := $WriteRef($t7, $t10);
    if (true) {
     $trace_temp := $Dereference(tpu);
      assume {:print "$track_local(7,16064,2):", $trace_temp} true;
    }

    // pack_ref($t7)

    // write_back[Reference(tpu)]($t7)
    call tpu := $WritebackToReference($t7, tpu);

    // $t11 := borrow_field<PackageTxnManager::TwoPhaseUpgrade>.upgrade_event(tpu)
    call $t11 := $BorrowField(tpu, $PackageTxnManager_TwoPhaseUpgrade_upgrade_event);

    // unpack_ref($t11)

    // $t12 := get_field<PackageTxnManager::UpgradePlan>.package_hash(plan)
    call $t12 := $GetFieldFromValue(plan, $PackageTxnManager_UpgradePlan_package_hash);

    // $t13 := get_field<PackageTxnManager::UpgradePlan>.version(plan)
    call $t13 := $GetFieldFromValue(plan, $PackageTxnManager_UpgradePlan_version);

    // $t14 := pack PackageTxnManager::UpgradeEvent($t3, $t12, $t13)
    call $t14 := $PackageTxnManager_UpgradeEvent_pack(0, 0, 0, $t3, $t12, $t13);

    // $t15 := read_ref($t11)
    call $t15 := $ReadRef($t11);
    assert $Event_EventHandle_$invariant_holds($t15);

    // $t15 := Event::emit_event<PackageTxnManager::UpgradeEvent>($t15, $t14)
    call $t15 := $Event_emit_event($PackageTxnManager_UpgradeEvent_type_value(), $t15, $t14);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,16184):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // write_ref($t11, $t15)
    call $t11 := $WriteRef($t11, $t15);
    if (true) {
     $trace_temp := $Dereference(tpu);
      assume {:print "$track_local(7,16184,2):", $trace_temp} true;
    }

    // pack_ref($t11)

    // write_back[Reference(tpu)]($t11)
    call tpu := $WritebackToReference($t11, tpu);

    // goto L2
    goto L2;

    // L2:
L2:

    // $t16 := Option::none<PackageTxnManager::UpgradePlan>()
    call $t16 := $Option_none_$direct_inter($PackageTxnManager_UpgradePlan_type_value());
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,16454):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t17 := borrow_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t17 := $BorrowField(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);

    // unpack_ref($t17)
    call $Option_Option_$unpack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t17));

    // write_ref($t17, $t16)
    call $t17 := $WriteRef($t17, $t16);
    if (true) {
     $trace_temp := $Dereference(tpu);
      assume {:print "$track_local(7,16435,2):", $trace_temp} true;
    }

    // pack_ref($t17)
    call $Option_Option_$pack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t17));

    // write_back[Reference(tpu)]($t17)
    call tpu := $WritebackToReference($t17, tpu);

    // pack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$pack_ref($Dereference(tpu));

    // write_back[PackageTxnManager::TwoPhaseUpgrade](tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$memory := $WritebackToGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, tpu);

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:timeLimit 40} $PackageTxnManager_finish_upgrade_plan_$verify(package_address: $Value) returns ()
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, package_address))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Option_spec_is_some($PackageTxnManager_UpgradePlan_type_value(), $SelectField($PackageTxnManager_tpu$23($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address), $PackageTxnManager_TwoPhaseUpgrade_plan))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($Config_Config_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Version_Version_type_value()], 1), $SelectField($SelectField($PackageTxnManager_tpu$23($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address), $PackageTxnManager_TwoPhaseUpgrade_version_cap), $Config_ModifyConfigCapability_account_address)))))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, package_address)))))
    || b#$Boolean(old($Boolean(b#$Boolean($Option_spec_is_some($PackageTxnManager_UpgradePlan_type_value(), $SelectField($PackageTxnManager_tpu$23($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address), $PackageTxnManager_TwoPhaseUpgrade_plan))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($Config_Config_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Version_Version_type_value()], 1), $SelectField($SelectField($PackageTxnManager_tpu$23($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address), $PackageTxnManager_TwoPhaseUpgrade_version_cap), $Config_ModifyConfigCapability_account_address)))))))));
{
    assume is#$Address(package_address);

    call $InitVerification();
    assume (forall $inv_addr: int, $inv_tv0: $TypeValue :: {contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr]}
        $Option_Option_$is_well_formed(contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr])
    );
    assume (forall $inv_addr: int, $inv_tv0: $TypeValue :: {contents#$Memory($Config_Config_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr]}
        $Config_Config_$is_well_formed(contents#$Memory($Config_Config_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr])
    );
    assume (forall $inv_addr: int, $inv_tv0: $TypeValue :: {contents#$Memory($Config_ModifyConfigCapability_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr]}
        $Config_ModifyConfigCapability_$is_well_formed(contents#$Memory($Config_ModifyConfigCapability_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_TwoPhaseUpgrade_$is_well_formed(contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_UpgradePlan_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_UpgradePlan_$is_well_formed(contents#$Memory($PackageTxnManager_UpgradePlan_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $PackageTxnManager_finish_upgrade_plan_$def_verify(package_address);
}


procedure {:inline 1} $PackageTxnManager_get_min_time_limit_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 100000
    $t0 := $Integer(100000);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,1274,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $PackageTxnManager_get_min_time_limit_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_min_time_limit_$def();
}


procedure {:inline 1} $PackageTxnManager_get_min_time_limit_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_min_time_limit_$def();
}


procedure {:inline 1} $PackageTxnManager_get_min_time_limit() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_min_time_limit_$def();
}


procedure {:inline 1} $PackageTxnManager_get_min_time_limit_$def_verify() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 100000
    $t0 := $Integer(100000);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,1274,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:timeLimit 40} $PackageTxnManager_get_min_time_limit_$verify() returns ($ret0: $Value)
ensures !$abort_flag;
{
    call $InitVerification();
    call $ret0 := $PackageTxnManager_get_min_time_limit_$def_verify();
}


procedure {:inline 1} $PackageTxnManager_get_module_upgrade_strategy_$def(module_address: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var tmp#$1: $Value; // $IntegerType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $PackageTxnManager_ModuleUpgradeStrategy_type_value()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := module_address;
      assume {:print "$track_local(7,11414,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(module_address)
    call $t2 := $CopyOrMoveValue(module_address);

    // $t3 := exists<PackageTxnManager::ModuleUpgradeStrategy>($t2)
    $t3 := $ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $t2);

    // if ($t3) goto L0 else goto L1
    if (b#$Boolean($t3)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t4 := get_global<PackageTxnManager::ModuleUpgradeStrategy>($t2)
    call $t4 := $GetGlobal($PackageTxnManager_ModuleUpgradeStrategy_$memory, $t2, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,11596):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t5 := get_field<PackageTxnManager::ModuleUpgradeStrategy>.strategy($t4)
    call $t5 := $GetFieldFromValue($t4, $PackageTxnManager_ModuleUpgradeStrategy_strategy);

    // tmp#$1 := $t5
    call tmp#$1 := $CopyOrMoveValue($t5);
    if (true) {
     $trace_temp := tmp#$1;
      assume {:print "$track_local(7,11527,1):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t6 := 0
    $t6 := $Integer(0);

    // tmp#$1 := $t6
    call tmp#$1 := $CopyOrMoveValue($t6);
    if (true) {
     $trace_temp := tmp#$1;
      assume {:print "$track_local(7,11527,1):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L3:
L3:

    // return tmp#$1
    $ret0 := tmp#$1;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,11527,7):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $PackageTxnManager_get_module_upgrade_strategy_$direct_inter(module_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(module_address);

    call $ret0 := $PackageTxnManager_get_module_upgrade_strategy_$def(module_address);
}


procedure {:inline 1} $PackageTxnManager_get_module_upgrade_strategy_$direct_intra(module_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(module_address);

    call $ret0 := $PackageTxnManager_get_module_upgrade_strategy_$def(module_address);
}


procedure {:inline 1} $PackageTxnManager_get_module_upgrade_strategy(module_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(module_address);

    call $ret0 := $PackageTxnManager_get_module_upgrade_strategy_$def(module_address);
}


procedure {:inline 1} $PackageTxnManager_get_module_upgrade_strategy_$def_verify(module_address: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var tmp#$1: $Value; // $IntegerType()
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $PackageTxnManager_ModuleUpgradeStrategy_type_value()
    var $t5: $Value; // $IntegerType()
    var $t6: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := module_address;
      assume {:print "$track_local(7,11414,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(module_address)
    call $t2 := $CopyOrMoveValue(module_address);

    // $t3 := exists<PackageTxnManager::ModuleUpgradeStrategy>($t2)
    $t3 := $ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $t2);

    // if ($t3) goto L0 else goto L1
    if (b#$Boolean($t3)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t4 := get_global<PackageTxnManager::ModuleUpgradeStrategy>($t2)
    call $t4 := $GetGlobal($PackageTxnManager_ModuleUpgradeStrategy_$memory, $t2, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,11596):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t5 := get_field<PackageTxnManager::ModuleUpgradeStrategy>.strategy($t4)
    call $t5 := $GetFieldFromValue($t4, $PackageTxnManager_ModuleUpgradeStrategy_strategy);

    // tmp#$1 := $t5
    call tmp#$1 := $CopyOrMoveValue($t5);
    if (true) {
     $trace_temp := tmp#$1;
      assume {:print "$track_local(7,11527,1):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t6 := 0
    $t6 := $Integer(0);

    // tmp#$1 := $t6
    call tmp#$1 := $CopyOrMoveValue($t6);
    if (true) {
     $trace_temp := tmp#$1;
      assume {:print "$track_local(7,11527,1):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L3:
L3:

    // return tmp#$1
    $ret0 := tmp#$1;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,11527,7):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:timeLimit 40} $PackageTxnManager_get_module_upgrade_strategy_$verify(module_address: $Value) returns ($ret0: $Value)
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
{
    assume is#$Address(module_address);

    call $InitVerification();
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_ModuleUpgradeStrategy_$is_well_formed(contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $ret0 := $PackageTxnManager_get_module_upgrade_strategy_$def_verify(module_address);
}


procedure {:inline 1} $PackageTxnManager_get_strategy_arbitrary_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0
    $t0 := $Integer(0);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,993,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $PackageTxnManager_get_strategy_arbitrary_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_arbitrary_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_arbitrary_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_arbitrary_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_arbitrary() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_arbitrary_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_arbitrary_$def_verify() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 0
    $t0 := $Integer(0);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,993,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:timeLimit 40} $PackageTxnManager_get_strategy_arbitrary_$verify() returns ($ret0: $Value)
ensures !$abort_flag;
{
    call $InitVerification();
    call $ret0 := $PackageTxnManager_get_strategy_arbitrary_$def_verify();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_freeze_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 3
    $t0 := $Integer(3);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,1208,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $PackageTxnManager_get_strategy_freeze_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_freeze_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_freeze_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_freeze_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_freeze() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_freeze_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_freeze_$def_verify() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 3
    $t0 := $Integer(3);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,1208,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:timeLimit 40} $PackageTxnManager_get_strategy_freeze_$verify() returns ($ret0: $Value)
ensures !$abort_flag;
{
    call $InitVerification();
    call $ret0 := $PackageTxnManager_get_strategy_freeze_$def_verify();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_new_module_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 2
    $t0 := $Integer(2);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,1138,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $PackageTxnManager_get_strategy_new_module_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_new_module_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_new_module_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_new_module_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_new_module() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_new_module_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_new_module_$def_verify() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 2
    $t0 := $Integer(2);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,1138,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:timeLimit 40} $PackageTxnManager_get_strategy_new_module_$verify() returns ($ret0: $Value)
ensures !$abort_flag;
{
    call $InitVerification();
    call $ret0 := $PackageTxnManager_get_strategy_new_module_$def_verify();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_two_phase_$def() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 1
    $t0 := $Integer(1);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,1065,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $PackageTxnManager_get_strategy_two_phase_$direct_inter() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_two_phase_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_two_phase_$direct_intra() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_two_phase_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_two_phase() returns ($ret0: $Value)
{
    call $ret0 := $PackageTxnManager_get_strategy_two_phase_$def();
}


procedure {:inline 1} $PackageTxnManager_get_strategy_two_phase_$def_verify() returns ($ret0: $Value)
{
    // declare local variables
    var $t0: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time

    // bytecode translation starts here
    // $t0 := 1
    $t0 := $Integer(1);

    // return $t0
    $ret0 := $t0;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,1065,1):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:timeLimit 40} $PackageTxnManager_get_strategy_two_phase_$verify() returns ($ret0: $Value)
ensures !$abort_flag;
{
    call $InitVerification();
    call $ret0 := $PackageTxnManager_get_strategy_two_phase_$def_verify();
}


procedure {:inline 1} $PackageTxnManager_get_upgrade_plan_$def(module_address: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var tmp#$1: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $PackageTxnManager_TwoPhaseUpgrade_type_value()
    var $t5: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := module_address;
      assume {:print "$track_local(7,12097,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(module_address)
    call $t2 := $CopyOrMoveValue(module_address);

    // $t3 := exists<PackageTxnManager::TwoPhaseUpgrade>($t2)
    $t3 := $ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $t2);

    // if ($t3) goto L0 else goto L1
    if (b#$Boolean($t3)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t4 := get_global<PackageTxnManager::TwoPhaseUpgrade>($t2)
    call $t4 := $GetGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, $t2, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,12275):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t5 := get_field<PackageTxnManager::TwoPhaseUpgrade>.plan($t4)
    call $t5 := $GetFieldFromValue($t4, $PackageTxnManager_TwoPhaseUpgrade_plan);

    // tmp#$1 := $t5
    call tmp#$1 := $CopyOrMoveValue($t5);
    if (true) {
     $trace_temp := tmp#$1;
      assume {:print "$track_local(7,12210,1):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L2:
L2:

    // tmp#$1 := Option::none<PackageTxnManager::UpgradePlan>()
    call tmp#$1 := $Option_none($PackageTxnManager_UpgradePlan_type_value());
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,12370):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // goto L3
    goto L3;

    // L3:
L3:

    // return tmp#$1
    $ret0 := tmp#$1;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,12210,6):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:inline 1} $PackageTxnManager_get_upgrade_plan_$direct_inter(module_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(module_address);

    call $ret0 := $PackageTxnManager_get_upgrade_plan_$def(module_address);
}


procedure {:inline 1} $PackageTxnManager_get_upgrade_plan_$direct_intra(module_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(module_address);

    call $ret0 := $PackageTxnManager_get_upgrade_plan_$def(module_address);
}


procedure {:inline 1} $PackageTxnManager_get_upgrade_plan(module_address: $Value) returns ($ret0: $Value)
{
    assume is#$Address(module_address);

    call $ret0 := $PackageTxnManager_get_upgrade_plan_$def(module_address);
}


procedure {:inline 1} $PackageTxnManager_get_upgrade_plan_$def_verify(module_address: $Value) returns ($ret0: $Value)
{
    // declare local variables
    var tmp#$1: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t2: $Value; // $AddressType()
    var $t3: $Value; // $BooleanType()
    var $t4: $Value; // $PackageTxnManager_TwoPhaseUpgrade_type_value()
    var $t5: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := module_address;
      assume {:print "$track_local(7,12097,0):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t2 := move(module_address)
    call $t2 := $CopyOrMoveValue(module_address);

    // $t3 := exists<PackageTxnManager::TwoPhaseUpgrade>($t2)
    $t3 := $ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $t2);

    // if ($t3) goto L0 else goto L1
    if (b#$Boolean($t3)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t4 := get_global<PackageTxnManager::TwoPhaseUpgrade>($t2)
    call $t4 := $GetGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, $t2, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,12275):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t5 := get_field<PackageTxnManager::TwoPhaseUpgrade>.plan($t4)
    call $t5 := $GetFieldFromValue($t4, $PackageTxnManager_TwoPhaseUpgrade_plan);

    // tmp#$1 := $t5
    call tmp#$1 := $CopyOrMoveValue($t5);
    if (true) {
     $trace_temp := tmp#$1;
      assume {:print "$track_local(7,12210,1):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L2:
L2:

    // tmp#$1 := Option::none<PackageTxnManager::UpgradePlan>()
    call tmp#$1 := $Option_none_$direct_inter($PackageTxnManager_UpgradePlan_type_value());
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,12370):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // goto L3
    goto L3;

    // L3:
L3:

    // return tmp#$1
    $ret0 := tmp#$1;
    if (true) {
     $trace_temp := $ret0;
      assume {:print "$track_local(7,12210,6):", $trace_temp} true;
    }
    return;

Abort:
    $abort_flag := true;
    $ret0 := $DefaultValue();
}

procedure {:timeLimit 40} $PackageTxnManager_get_upgrade_plan_$verify(module_address: $Value) returns ($ret0: $Value)
ensures b#$Boolean(old($Boolean(false))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(false))));
{
    assume is#$Address(module_address);

    call $InitVerification();
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_TwoPhaseUpgrade_$is_well_formed(contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $ret0 := $PackageTxnManager_get_upgrade_plan_$def_verify(module_address);
}


procedure {:inline 1} $PackageTxnManager_package_txn_epilogue_$def(account: $Value, _txn_sender: $Value, package_address: $Value, success: $Value) returns ()
{
    // declare local variables
    var strategy: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $BooleanType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,17398,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := _txn_sender;
      assume {:print "$track_local(7,17398,1):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_address;
      assume {:print "$track_local(7,17398,2):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := success;
      assume {:print "$track_local(7,17398,3):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t5 := move(account)
    call $t5 := $CopyOrMoveValue(account);

    // $t6 := move(package_address)
    call $t6 := $CopyOrMoveValue(package_address);

    // $t7 := move(success)
    call $t7 := $CopyOrMoveValue(success);

    // CoreAddresses::assert_genesis_address($t5)
    call $CoreAddresses_assert_genesis_address($t5);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,17642):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // strategy := PackageTxnManager::get_module_upgrade_strategy($t6)
    call strategy := $PackageTxnManager_get_module_upgrade_strategy($t6);
    if ($abort_flag) {
      goto Abort;
    }

    // $t8 := 1
    $t8 := $Integer(1);

    // $t9 := ==(strategy, $t8)
    $t9 := $Boolean($IsEqual(strategy, $t8));

    // if ($t9) goto L0 else goto L1
    if (b#$Boolean($t9)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // if ($t7) goto L3 else goto L4
    if (b#$Boolean($t7)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L2
    goto L2;

    // L3:
L3:

    // PackageTxnManager::finish_upgrade_plan($t6)
    call $PackageTxnManager_finish_upgrade_plan($t6);
    if ($abort_flag) {
      goto Abort;
    }

    // goto L2
    goto L2;

    // L2:
L2:

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $PackageTxnManager_package_txn_epilogue_$direct_inter(account: $Value, _txn_sender: $Value, package_address: $Value, success: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(_txn_sender);

    assume is#$Address(package_address);

    assume is#$Boolean(success);

    call $PackageTxnManager_package_txn_epilogue_$def(account, _txn_sender, package_address, success);
}


procedure {:inline 1} $PackageTxnManager_package_txn_epilogue_$direct_intra(account: $Value, _txn_sender: $Value, package_address: $Value, success: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(_txn_sender);

    assume is#$Address(package_address);

    assume is#$Boolean(success);

    call $PackageTxnManager_package_txn_epilogue_$def(account, _txn_sender, package_address, success);
}


procedure {:inline 1} $PackageTxnManager_package_txn_epilogue(account: $Value, _txn_sender: $Value, package_address: $Value, success: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(_txn_sender);

    assume is#$Address(package_address);

    assume is#$Boolean(success);

    call $PackageTxnManager_package_txn_epilogue_$def(account, _txn_sender, package_address, success);
}


procedure {:inline 1} $PackageTxnManager_package_txn_epilogue_$def_verify(account: $Value, _txn_sender: $Value, package_address: $Value, success: $Value) returns ()
{
    // declare local variables
    var strategy: $Value; // $IntegerType()
    var $t5: $Value; // $AddressType()
    var $t6: $Value; // $AddressType()
    var $t7: $Value; // $BooleanType()
    var $t8: $Value; // $IntegerType()
    var $t9: $Value; // $BooleanType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,17398,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := _txn_sender;
      assume {:print "$track_local(7,17398,1):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_address;
      assume {:print "$track_local(7,17398,2):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := success;
      assume {:print "$track_local(7,17398,3):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t5 := move(account)
    call $t5 := $CopyOrMoveValue(account);

    // $t6 := move(package_address)
    call $t6 := $CopyOrMoveValue(package_address);

    // $t7 := move(success)
    call $t7 := $CopyOrMoveValue(success);

    // CoreAddresses::assert_genesis_address($t5)
    call $CoreAddresses_assert_genesis_address_$direct_inter($t5);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,17642):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // strategy := PackageTxnManager::get_module_upgrade_strategy($t6)
    call strategy := $PackageTxnManager_get_module_upgrade_strategy_$direct_intra($t6);
    if ($abort_flag) {
      goto Abort;
    }

    // $t8 := 1
    $t8 := $Integer(1);

    // $t9 := ==(strategy, $t8)
    $t9 := $Boolean($IsEqual(strategy, $t8));

    // if ($t9) goto L0 else goto L1
    if (b#$Boolean($t9)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // if ($t7) goto L3 else goto L4
    if (b#$Boolean($t7)) { goto L3; } else { goto L4; }

    // L4:
L4:

    // goto L2
    goto L2;

    // L3:
L3:

    // PackageTxnManager::finish_upgrade_plan($t6)
    call $PackageTxnManager_finish_upgrade_plan_$direct_intra($t6);
    if ($abort_flag) {
      goto Abort;
    }

    // goto L2
    goto L2;

    // L2:
L2:

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:timeLimit 40} $PackageTxnManager_package_txn_epilogue_$verify(account: $Value, _txn_sender: $Value, package_address: $Value, success: $Value) returns ()
ensures b#$Boolean(old($Boolean(!$IsEqual($Signer_$address_of(account), $CoreAddresses_SPEC_GENESIS_ADDRESS())))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean(success))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, package_address))))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean(success))) && b#$Boolean($Option_spec_is_some($PackageTxnManager_UpgradePlan_type_value(), $SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, package_address), $PackageTxnManager_TwoPhaseUpgrade_plan))))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($Config_Config_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Version_Version_type_value()], 1), $SelectField($SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, package_address), $PackageTxnManager_TwoPhaseUpgrade_version_cap), $Config_ModifyConfigCapability_account_address)))))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!$IsEqual($Signer_$address_of(account), $CoreAddresses_SPEC_GENESIS_ADDRESS()))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean(success))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, package_address)))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean(success))) && b#$Boolean($Option_spec_is_some($PackageTxnManager_UpgradePlan_type_value(), $SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, package_address), $PackageTxnManager_TwoPhaseUpgrade_plan))))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($Config_Config_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Version_Version_type_value()], 1), $SelectField($SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, package_address), $PackageTxnManager_TwoPhaseUpgrade_version_cap), $Config_ModifyConfigCapability_account_address)))))))));
{
    assume is#$Address(account);

    assume is#$Address(_txn_sender);

    assume is#$Address(package_address);

    assume is#$Boolean(success);

    call $InitVerification();
    assume (forall $inv_addr: int, $inv_tv0: $TypeValue :: {contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr]}
        $Option_Option_$is_well_formed(contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr])
    );
    assume (forall $inv_addr: int, $inv_tv0: $TypeValue :: {contents#$Memory($Config_Config_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr]}
        $Config_Config_$is_well_formed(contents#$Memory($Config_Config_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr])
    );
    assume (forall $inv_addr: int, $inv_tv0: $TypeValue :: {contents#$Memory($Config_ModifyConfigCapability_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr]}
        $Config_ModifyConfigCapability_$is_well_formed(contents#$Memory($Config_ModifyConfigCapability_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_ModuleUpgradeStrategy_$is_well_formed(contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_TwoPhaseUpgrade_$is_well_formed(contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_UpgradePlan_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_UpgradePlan_$is_well_formed(contents#$Memory($PackageTxnManager_UpgradePlan_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $PackageTxnManager_package_txn_epilogue_$def_verify(account, _txn_sender, package_address, success);
}


procedure {:inline 1} $PackageTxnManager_package_txn_prologue_$def(account: $Value, package_address: $Value, package_hash: $Value) returns ()
{
    // declare local variables
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $Vector_type_value($IntegerType())
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,16805,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_address;
      assume {:print "$track_local(7,16805,1):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_hash;
      assume {:print "$track_local(7,16805,2):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(account)
    call $t3 := $CopyOrMoveValue(account);

    // $t4 := move(package_address)
    call $t4 := $CopyOrMoveValue(package_address);

    // $t5 := move(package_hash)
    call $t5 := $CopyOrMoveValue(package_hash);

    // CoreAddresses::assert_genesis_address($t3)
    call $CoreAddresses_assert_genesis_address($t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,17038):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // PackageTxnManager::check_package_txn($t4, $t5)
    call $PackageTxnManager_check_package_txn($t4, $t5);
    if ($abort_flag) {
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $PackageTxnManager_package_txn_prologue_$direct_inter(account: $Value, package_address: $Value, package_hash: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(package_address);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    call $PackageTxnManager_package_txn_prologue_$def(account, package_address, package_hash);
}


procedure {:inline 1} $PackageTxnManager_package_txn_prologue_$direct_intra(account: $Value, package_address: $Value, package_hash: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(package_address);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    call $PackageTxnManager_package_txn_prologue_$def(account, package_address, package_hash);
}


procedure {:inline 1} $PackageTxnManager_package_txn_prologue(account: $Value, package_address: $Value, package_hash: $Value) returns ()
{
    assume is#$Address(account);

    assume is#$Address(package_address);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    call $PackageTxnManager_package_txn_prologue_$def(account, package_address, package_hash);
}


procedure {:inline 1} $PackageTxnManager_package_txn_prologue_$def_verify(account: $Value, package_address: $Value, package_hash: $Value) returns ()
{
    // declare local variables
    var $t3: $Value; // $AddressType()
    var $t4: $Value; // $AddressType()
    var $t5: $Value; // $Vector_type_value($IntegerType())
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,16805,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_address;
      assume {:print "$track_local(7,16805,1):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_hash;
      assume {:print "$track_local(7,16805,2):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t3 := move(account)
    call $t3 := $CopyOrMoveValue(account);

    // $t4 := move(package_address)
    call $t4 := $CopyOrMoveValue(package_address);

    // $t5 := move(package_hash)
    call $t5 := $CopyOrMoveValue(package_hash);

    // CoreAddresses::assert_genesis_address($t3)
    call $CoreAddresses_assert_genesis_address_$direct_inter($t3);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,17038):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // PackageTxnManager::check_package_txn($t4, $t5)
    call $PackageTxnManager_check_package_txn_$direct_intra($t4, $t5);
    if ($abort_flag) {
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:timeLimit 40} $PackageTxnManager_package_txn_prologue_$verify(account: $Value, package_address: $Value, package_hash: $Value) returns ()
ensures b#$Boolean(old($Boolean(!$IsEqual($Signer_$address_of(account), $CoreAddresses_SPEC_GENESIS_ADDRESS())))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(3))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Option_spec_is_none($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(!$IsEqual($SelectField($Option_spec_get($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address)), $PackageTxnManager_UpgradePlan_package_hash), package_hash)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS()))))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(i#$Integer($SelectField($Option_spec_get($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address)), $PackageTxnManager_UpgradePlan_active_after_time)) > i#$Integer($Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory))))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!$IsEqual($Signer_$address_of(account), $CoreAddresses_SPEC_GENESIS_ADDRESS()))))
    || b#$Boolean(old($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(3)))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Option_spec_is_none($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(!$IsEqual($SelectField($Option_spec_get($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address)), $PackageTxnManager_UpgradePlan_package_hash), package_hash))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS())))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual($PackageTxnManager_spec_get_module_upgrade_strategy($PackageTxnManager_ModuleUpgradeStrategy_$memory, package_address), $Integer(1)))) && b#$Boolean($Boolean(i#$Integer($SelectField($Option_spec_get($PackageTxnManager_UpgradePlan_type_value(), $PackageTxnManager_spec_get_upgrade_plan($PackageTxnManager_TwoPhaseUpgrade_$memory, package_address)), $PackageTxnManager_UpgradePlan_active_after_time)) > i#$Integer($Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory))))))));
{
    assume is#$Address(account);

    assume is#$Address(package_address);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    call $InitVerification();
    assume (forall $inv_addr: int, $inv_tv0: $TypeValue :: {contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr]}
        $Option_Option_$is_well_formed(contents#$Memory($Option_Option_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($Timestamp_CurrentTimeMilliseconds_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $Timestamp_CurrentTimeMilliseconds_$is_well_formed(contents#$Memory($Timestamp_CurrentTimeMilliseconds_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_ModuleUpgradeStrategy_$is_well_formed(contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_TwoPhaseUpgrade_$is_well_formed(contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_UpgradePlan_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_UpgradePlan_$is_well_formed(contents#$Memory($PackageTxnManager_UpgradePlan_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $PackageTxnManager_package_txn_prologue_$def_verify(account, package_address, package_hash);
}


procedure {:inline 1} $PackageTxnManager_submit_upgrade_plan_$def(account: $Value, package_hash: $Value, version: $Value, min_milliseconds: $Value) returns ()
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var active_after_time: $Value; // $IntegerType()
    var cap: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var tmp#$7: $Value; // $BooleanType()
    var tmp#$8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $Vector_type_value($IntegerType())
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,6677,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_hash;
      assume {:print "$track_local(7,6677,1):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := version;
      assume {:print "$track_local(7,6677,2):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := min_milliseconds;
      assume {:print "$track_local(7,6677,3):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t9 := move(account)
    call $t9 := $CopyOrMoveValue(account);

    // $t10 := move(package_hash)
    call $t10 := $CopyOrMoveValue(package_hash);

    // $t11 := move(version)
    call $t11 := $CopyOrMoveValue(version);

    // $t12 := move(min_milliseconds)
    call $t12 := $CopyOrMoveValue(min_milliseconds);

    // $t13 := 100000
    $t13 := $Integer(100000);

    // $t14 := >=($t12, $t13)
    call $t14 := $Ge($t12, $t13);

    // $t15 := 104
    $t15 := $Integer(104);

    // $t16 := Errors::invalid_argument($t15)
    call $t16 := $Errors_invalid_argument($t15);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6921):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t14) goto L0 else goto L1
    if (b#$Boolean($t14)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // destroy($t9)

    // abort($t16)
    $abort_code := i#$Integer($t16);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6870):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // account_address := Signer::address_of($t9)
    call account_address := $Signer_address_of($t9);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,7006):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // cap := get_global<PackageTxnManager::UpgradePlanCapability>(account_address)
    call cap := $GetGlobal($PackageTxnManager_UpgradePlanCapability_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,7049):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,7049,6):", $trace_temp} true;
    }

    // $t17 := Timestamp::now_milliseconds()
    call $t17 := $Timestamp_now_milliseconds();
    if ($abort_flag) {
      goto Abort;
    }

    // active_after_time := +($t17, $t12)
    call active_after_time := $AddU64($t17, $t12);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,7170):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := active_after_time;
      assume {:print "$track_local(7,7170,5):", $trace_temp} true;
    }

    // PackageTxnManager::submit_upgrade_plan_with_cap(cap, $t10, $t11, active_after_time)
    call $PackageTxnManager_submit_upgrade_plan_with_cap(cap, $t10, $t11, active_after_time);
    if ($abort_flag) {
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $PackageTxnManager_submit_upgrade_plan_$direct_inter(account: $Value, package_hash: $Value, version: $Value, min_milliseconds: $Value) returns ()
{
    assume is#$Address(account);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    assume $IsValidU64(version);

    assume $IsValidU64(min_milliseconds);

    call $PackageTxnManager_submit_upgrade_plan_$def(account, package_hash, version, min_milliseconds);
}


procedure {:inline 1} $PackageTxnManager_submit_upgrade_plan_$direct_intra(account: $Value, package_hash: $Value, version: $Value, min_milliseconds: $Value) returns ()
{
    assume is#$Address(account);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    assume $IsValidU64(version);

    assume $IsValidU64(min_milliseconds);

    call $PackageTxnManager_submit_upgrade_plan_$def(account, package_hash, version, min_milliseconds);
}


procedure {:inline 1} $PackageTxnManager_submit_upgrade_plan(account: $Value, package_hash: $Value, version: $Value, min_milliseconds: $Value) returns ()
{
    assume is#$Address(account);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    assume $IsValidU64(version);

    assume $IsValidU64(min_milliseconds);

    call $PackageTxnManager_submit_upgrade_plan_$def(account, package_hash, version, min_milliseconds);
}


procedure {:inline 1} $PackageTxnManager_submit_upgrade_plan_$def_verify(account: $Value, package_hash: $Value, version: $Value, min_milliseconds: $Value) returns ()
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var active_after_time: $Value; // $IntegerType()
    var cap: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var tmp#$7: $Value; // $BooleanType()
    var tmp#$8: $Value; // $IntegerType()
    var $t9: $Value; // $AddressType()
    var $t10: $Value; // $Vector_type_value($IntegerType())
    var $t11: $Value; // $IntegerType()
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $BooleanType()
    var $t15: $Value; // $IntegerType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,6677,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_hash;
      assume {:print "$track_local(7,6677,1):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := version;
      assume {:print "$track_local(7,6677,2):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := min_milliseconds;
      assume {:print "$track_local(7,6677,3):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t9 := move(account)
    call $t9 := $CopyOrMoveValue(account);

    // $t10 := move(package_hash)
    call $t10 := $CopyOrMoveValue(package_hash);

    // $t11 := move(version)
    call $t11 := $CopyOrMoveValue(version);

    // $t12 := move(min_milliseconds)
    call $t12 := $CopyOrMoveValue(min_milliseconds);

    // $t13 := 100000
    $t13 := $Integer(100000);

    // $t14 := >=($t12, $t13)
    call $t14 := $Ge($t12, $t13);

    // $t15 := 104
    $t15 := $Integer(104);

    // $t16 := Errors::invalid_argument($t15)
    call $t16 := $Errors_invalid_argument_$direct_inter($t15);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6921):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t14) goto L0 else goto L1
    if (b#$Boolean($t14)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // destroy($t9)

    // abort($t16)
    $abort_code := i#$Integer($t16);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,6870):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // account_address := Signer::address_of($t9)
    call account_address := $Signer_address_of_$direct_inter($t9);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,7006):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // cap := get_global<PackageTxnManager::UpgradePlanCapability>(account_address)
    call cap := $GetGlobal($PackageTxnManager_UpgradePlanCapability_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,7049):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,7049,6):", $trace_temp} true;
    }

    // $t17 := Timestamp::now_milliseconds()
    call $t17 := $Timestamp_now_milliseconds_$direct_inter();
    if ($abort_flag) {
      goto Abort;
    }

    // active_after_time := +($t17, $t12)
    call active_after_time := $AddU64($t17, $t12);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,7170):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := active_after_time;
      assume {:print "$track_local(7,7170,5):", $trace_temp} true;
    }

    // PackageTxnManager::submit_upgrade_plan_with_cap(cap, $t10, $t11, active_after_time)
    call $PackageTxnManager_submit_upgrade_plan_with_cap_$direct_intra(cap, $t10, $t11, active_after_time);
    if ($abort_flag) {
      goto Abort;
    }

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:timeLimit 40} $PackageTxnManager_submit_upgrade_plan_$verify(account: $Value, package_hash: $Value, version: $Value, min_milliseconds: $Value) returns ()
ensures b#$Boolean(old($Boolean(i#$Integer(min_milliseconds) < i#$Integer($Integer(100000))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS()))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(i#$Integer($Integer(i#$Integer($Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory)) + i#$Integer(min_milliseconds))) > i#$Integer($Integer($MAX_U64))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS()))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(i#$Integer($PackageTxnManager_active_after_time$22($Timestamp_CurrentTimeMilliseconds_$memory, min_milliseconds)) < i#$Integer($Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!$IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(i#$Integer(min_milliseconds) < i#$Integer($Integer(100000)))))
    || b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account))))))
    || b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS())))))
    || b#$Boolean(old($Boolean(i#$Integer($Integer(i#$Integer($Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory)) + i#$Integer(min_milliseconds))) > i#$Integer($Integer($MAX_U64)))))
    || b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS())))))
    || b#$Boolean(old($Boolean(i#$Integer($PackageTxnManager_active_after_time$22($Timestamp_CurrentTimeMilliseconds_$memory, min_milliseconds)) < i#$Integer($Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory)))))
    || b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address))))))
    || b#$Boolean(old($Boolean(!$IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1)))))
    || b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)))))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_some($PackageTxnManager_UpgradePlan_type_value(), $SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField($ResourceValue($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_TwoPhaseUpgrade_plan))));
{
    assume is#$Address(account);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    assume $IsValidU64(version);

    assume $IsValidU64(min_milliseconds);

    call $InitVerification();
    assume (forall $inv_addr: int :: {contents#$Memory($Timestamp_CurrentTimeMilliseconds_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $Timestamp_CurrentTimeMilliseconds_$is_well_formed(contents#$Memory($Timestamp_CurrentTimeMilliseconds_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_ModuleUpgradeStrategy_$is_well_formed(contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_TwoPhaseUpgrade_$is_well_formed(contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_UpgradePlanCapability_$is_well_formed(contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $PackageTxnManager_submit_upgrade_plan_$def_verify(account, package_hash, version, min_milliseconds);
}


procedure {:inline 1} $PackageTxnManager_submit_upgrade_plan_with_cap_$def(cap: $Value, package_hash: $Value, version: $Value, active_after_time: $Value) returns ()
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var tmp#$5: $Value; // $BooleanType()
    var tmp#$6: $Value; // $IntegerType()
    var tmp#$7: $Value; // $BooleanType()
    var tmp#$8: $Value; // $IntegerType()
    var tpu: $Mutation; // ReferenceType($PackageTxnManager_TwoPhaseUpgrade_type_value())
    var $t10: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t11: $Value; // $Vector_type_value($IntegerType())
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $BooleanType()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $PackageTxnManager_UpgradePlan_type_value()
    var $t25: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t26: $Mutation; // ReferenceType($Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value()))
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,8065,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_hash;
      assume {:print "$track_local(7,8065,1):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := version;
      assume {:print "$track_local(7,8065,2):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := active_after_time;
      assume {:print "$track_local(7,8065,3):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t10 := move(cap)
    call $t10 := $CopyOrMoveValue(cap);

    // $t11 := move(package_hash)
    call $t11 := $CopyOrMoveValue(package_hash);

    // $t12 := move(version)
    call $t12 := $CopyOrMoveValue(version);

    // $t13 := move(active_after_time)
    call $t13 := $CopyOrMoveValue(active_after_time);

    // $t14 := Timestamp::now_milliseconds()
    call $t14 := $Timestamp_now_milliseconds();
    if ($abort_flag) {
      goto Abort;
    }

    // $t15 := >=($t13, $t14)
    call $t15 := $Ge($t13, $t14);

    // $t16 := 104
    $t16 := $Integer(104);

    // $t17 := Errors::invalid_argument($t16)
    call $t17 := $Errors_invalid_argument($t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,8325):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t15) goto L0 else goto L1
    if (b#$Boolean($t15)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // destroy($t10)

    // abort($t17)
    $abort_code := i#$Integer($t17);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,8258):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // $t18 := get_field<PackageTxnManager::UpgradePlanCapability>.account_address($t10)
    call $t18 := $GetFieldFromValue($t10, $PackageTxnManager_UpgradePlanCapability_account_address);

    // account_address := $t18
    call account_address := $CopyOrMoveValue($t18);
    if (true) {
     $trace_temp := account_address;
      assume {:print "$track_local(7,8384,4):", $trace_temp} true;
    }

    // $t19 := PackageTxnManager::get_module_upgrade_strategy(account_address)
    call $t19 := $PackageTxnManager_get_module_upgrade_strategy(account_address);
    if ($abort_flag) {
      goto Abort;
    }

    // $t20 := 1
    $t20 := $Integer(1);

    // $t21 := ==($t19, $t20)
    $t21 := $Boolean($IsEqual($t19, $t20));

    // $t22 := 107
    $t22 := $Integer(107);

    // $t23 := Errors::invalid_argument($t22)
    call $t23 := $Errors_invalid_argument($t22);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,8518):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t21) goto L2 else goto L3
    if (b#$Boolean($t21)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // abort($t23)
    $abort_code := i#$Integer($t23);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,8435):", $trace_abort_temp} true;
    }
    goto Abort;

    // L2:
L2:

    // tpu := borrow_global<PackageTxnManager::TwoPhaseUpgrade>(account_address)
    call tpu := $BorrowGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,8584):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$unpack_ref($Dereference(tpu));

    // $t24 := pack PackageTxnManager::UpgradePlan($t11, $t13, $t12)
    call $t24 := $PackageTxnManager_UpgradePlan_pack(0, 0, 0, $t11, $t13, $t12);

    // $t25 := Option::some<PackageTxnManager::UpgradePlan>($t24)
    call $t25 := $Option_some($PackageTxnManager_UpgradePlan_type_value(), $t24);
    if ($abort_flag) {
      goto Abort;
    }

    // $t26 := borrow_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t26 := $BorrowField(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);

    // unpack_ref($t26)
    call $Option_Option_$unpack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t26));

    // write_ref($t26, $t25)
    call $t26 := $WriteRef($t26, $t25);
    if (true) {
     $trace_temp := $Dereference(tpu);
      assume {:print "$track_local(7,8649,9):", $trace_temp} true;
    }

    // pack_ref($t26)
    call $Option_Option_$pack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t26));

    // write_back[Reference(tpu)]($t26)
    call tpu := $WritebackToReference($t26, tpu);

    // pack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$pack_ref($Dereference(tpu));

    // write_back[PackageTxnManager::TwoPhaseUpgrade](tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$memory := $WritebackToGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, tpu);

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $PackageTxnManager_submit_upgrade_plan_with_cap_$direct_inter(cap: $Value, package_hash: $Value, version: $Value, active_after_time: $Value) returns ()
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    assume $IsValidU64(version);

    assume $IsValidU64(active_after_time);

    call $PackageTxnManager_submit_upgrade_plan_with_cap_$def(cap, package_hash, version, active_after_time);
}


procedure {:inline 1} $PackageTxnManager_submit_upgrade_plan_with_cap_$direct_intra(cap: $Value, package_hash: $Value, version: $Value, active_after_time: $Value) returns ()
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    assume $IsValidU64(version);

    assume $IsValidU64(active_after_time);

    call $PackageTxnManager_submit_upgrade_plan_with_cap_$def(cap, package_hash, version, active_after_time);
}


procedure {:inline 1} $PackageTxnManager_submit_upgrade_plan_with_cap(cap: $Value, package_hash: $Value, version: $Value, active_after_time: $Value) returns ()
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    assume $IsValidU64(version);

    assume $IsValidU64(active_after_time);

    call $PackageTxnManager_submit_upgrade_plan_with_cap_$def(cap, package_hash, version, active_after_time);
}


procedure {:inline 1} $PackageTxnManager_submit_upgrade_plan_with_cap_$def_verify(cap: $Value, package_hash: $Value, version: $Value, active_after_time: $Value) returns ()
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var tmp#$5: $Value; // $BooleanType()
    var tmp#$6: $Value; // $IntegerType()
    var tmp#$7: $Value; // $BooleanType()
    var tmp#$8: $Value; // $IntegerType()
    var tpu: $Mutation; // ReferenceType($PackageTxnManager_TwoPhaseUpgrade_type_value())
    var $t10: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t11: $Value; // $Vector_type_value($IntegerType())
    var $t12: $Value; // $IntegerType()
    var $t13: $Value; // $IntegerType()
    var $t14: $Value; // $IntegerType()
    var $t15: $Value; // $BooleanType()
    var $t16: $Value; // $IntegerType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $AddressType()
    var $t19: $Value; // $IntegerType()
    var $t20: $Value; // $IntegerType()
    var $t21: $Value; // $BooleanType()
    var $t22: $Value; // $IntegerType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $PackageTxnManager_UpgradePlan_type_value()
    var $t25: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t26: $Mutation; // ReferenceType($Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value()))
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,8065,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := package_hash;
      assume {:print "$track_local(7,8065,1):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := version;
      assume {:print "$track_local(7,8065,2):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := active_after_time;
      assume {:print "$track_local(7,8065,3):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t10 := move(cap)
    call $t10 := $CopyOrMoveValue(cap);

    // $t11 := move(package_hash)
    call $t11 := $CopyOrMoveValue(package_hash);

    // $t12 := move(version)
    call $t12 := $CopyOrMoveValue(version);

    // $t13 := move(active_after_time)
    call $t13 := $CopyOrMoveValue(active_after_time);

    // $t14 := Timestamp::now_milliseconds()
    call $t14 := $Timestamp_now_milliseconds_$direct_inter();
    if ($abort_flag) {
      goto Abort;
    }

    // $t15 := >=($t13, $t14)
    call $t15 := $Ge($t13, $t14);

    // $t16 := 104
    $t16 := $Integer(104);

    // $t17 := Errors::invalid_argument($t16)
    call $t17 := $Errors_invalid_argument_$direct_inter($t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,8325):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t15) goto L0 else goto L1
    if (b#$Boolean($t15)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // destroy($t10)

    // abort($t17)
    $abort_code := i#$Integer($t17);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,8258):", $trace_abort_temp} true;
    }
    goto Abort;

    // L0:
L0:

    // $t18 := get_field<PackageTxnManager::UpgradePlanCapability>.account_address($t10)
    call $t18 := $GetFieldFromValue($t10, $PackageTxnManager_UpgradePlanCapability_account_address);

    // account_address := $t18
    call account_address := $CopyOrMoveValue($t18);
    if (true) {
     $trace_temp := account_address;
      assume {:print "$track_local(7,8384,4):", $trace_temp} true;
    }

    // $t19 := PackageTxnManager::get_module_upgrade_strategy(account_address)
    call $t19 := $PackageTxnManager_get_module_upgrade_strategy_$direct_intra(account_address);
    if ($abort_flag) {
      goto Abort;
    }

    // $t20 := 1
    $t20 := $Integer(1);

    // $t21 := ==($t19, $t20)
    $t21 := $Boolean($IsEqual($t19, $t20));

    // $t22 := 107
    $t22 := $Integer(107);

    // $t23 := Errors::invalid_argument($t22)
    call $t23 := $Errors_invalid_argument_$direct_inter($t22);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,8518):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t21) goto L2 else goto L3
    if (b#$Boolean($t21)) { goto L2; } else { goto L3; }

    // L3:
L3:

    // abort($t23)
    $abort_code := i#$Integer($t23);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,8435):", $trace_abort_temp} true;
    }
    goto Abort;

    // L2:
L2:

    // tpu := borrow_global<PackageTxnManager::TwoPhaseUpgrade>(account_address)
    call tpu := $BorrowGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,8584):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$unpack_ref($Dereference(tpu));

    // $t24 := pack PackageTxnManager::UpgradePlan($t11, $t13, $t12)
    call $t24 := $PackageTxnManager_UpgradePlan_pack(0, 0, 0, $t11, $t13, $t12);

    // $t25 := Option::some<PackageTxnManager::UpgradePlan>($t24)
    call $t25 := $Option_some_$direct_inter($PackageTxnManager_UpgradePlan_type_value(), $t24);
    if ($abort_flag) {
      goto Abort;
    }

    // $t26 := borrow_field<PackageTxnManager::TwoPhaseUpgrade>.plan(tpu)
    call $t26 := $BorrowField(tpu, $PackageTxnManager_TwoPhaseUpgrade_plan);

    // unpack_ref($t26)
    call $Option_Option_$unpack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t26));

    // write_ref($t26, $t25)
    call $t26 := $WriteRef($t26, $t25);
    if (true) {
     $trace_temp := $Dereference(tpu);
      assume {:print "$track_local(7,8649,9):", $trace_temp} true;
    }

    // pack_ref($t26)
    call $Option_Option_$pack_ref($PackageTxnManager_UpgradePlan_type_value(), $Dereference($t26));

    // write_back[Reference(tpu)]($t26)
    call tpu := $WritebackToReference($t26, tpu);

    // pack_ref(tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$pack_ref($Dereference(tpu));

    // write_back[PackageTxnManager::TwoPhaseUpgrade](tpu)
    call $PackageTxnManager_TwoPhaseUpgrade_$memory := $WritebackToGlobal($PackageTxnManager_TwoPhaseUpgrade_$memory, tpu);

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:timeLimit 40} $PackageTxnManager_submit_upgrade_plan_with_cap_$verify(cap: $Value, package_hash: $Value, version: $Value, active_after_time: $Value) returns ()
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS()))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(i#$Integer(active_after_time) < i#$Integer($Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!$IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($Timestamp_CurrentTimeMilliseconds_$memory, $EmptyTypeValueArray, $CoreAddresses_$GENESIS_ADDRESS())))))
    || b#$Boolean(old($Boolean(i#$Integer(active_after_time) < i#$Integer($Timestamp_$now_milliseconds($Timestamp_CurrentTimeMilliseconds_$memory)))))
    || b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address))))))
    || b#$Boolean(old($Boolean(!$IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1)))))
    || b#$Boolean(old($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)))))));
ensures !$abort_flag ==> (b#$Boolean($Option_spec_is_some($PackageTxnManager_UpgradePlan_type_value(), $SelectField($ResourceValue($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $SelectField(cap, $PackageTxnManager_UpgradePlanCapability_account_address)), $PackageTxnManager_TwoPhaseUpgrade_plan))));
{
    assume $PackageTxnManager_UpgradePlanCapability_$is_well_formed(cap);

    assume $Vector_$is_well_formed(package_hash) && (forall $$0: int :: {$select_vector(package_hash,$$0)} $$0 >= 0 && $$0 < $vlen(package_hash) ==> $IsValidU8($select_vector(package_hash,$$0)));

    assume $IsValidU64(version);

    assume $IsValidU64(active_after_time);

    call $InitVerification();
    assume (forall $inv_addr: int :: {contents#$Memory($Timestamp_CurrentTimeMilliseconds_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $Timestamp_CurrentTimeMilliseconds_$is_well_formed(contents#$Memory($Timestamp_CurrentTimeMilliseconds_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_ModuleUpgradeStrategy_$is_well_formed(contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_TwoPhaseUpgrade_$is_well_formed(contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_UpgradePlanCapability_$is_well_formed(contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $PackageTxnManager_submit_upgrade_plan_with_cap_$def_verify(cap, package_hash, version, active_after_time);
}


procedure {:inline 1} $PackageTxnManager_update_module_upgrade_strategy_$def(account: $Value, strategy: $Value) returns ()
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var cap: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var previous_strategy: $Value; // $IntegerType()
    var tmp#$5: $Value; // $BooleanType()
    var tmp#$6: $Value; // $IntegerType()
    var tmp#$7: $Value; // $BooleanType()
    var tmp#$8: $Value; // $BooleanType()
    var tmp#$9: $Value; // $BooleanType()
    var tmp#$10: $Value; // $BooleanType()
    var tmp#$11: $Value; // $IntegerType()
    var tpu: $Value; // $PackageTxnManager_TwoPhaseUpgrade_type_value()
    var upgrade_event: $Value; // $Event_EventHandle_type_value($PackageTxnManager_UpgradeEvent_type_value())
    var version_cap: $Value; // $Config_ModifyConfigCapability_type_value($Version_Version_type_value())
    var version_cap#452: $Value; // $Config_ModifyConfigCapability_type_value($Version_Version_type_value())
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $IntegerType()
    var $t22: $Value; // $BooleanType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $BooleanType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $IntegerType()
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $BooleanType()
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $IntegerType()
    var $t31: $Value; // $BooleanType()
    var $t32: $Mutation; // ReferenceType($PackageTxnManager_ModuleUpgradeStrategy_type_value())
    var $t33: $Mutation; // ReferenceType($IntegerType())
    var $t34: $Value; // $PackageTxnManager_ModuleUpgradeStrategy_type_value()
    var $t35: $Value; // $IntegerType()
    var $t36: $Value; // $BooleanType()
    var $t37: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t38: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t39: $Value; // $Event_EventHandle_type_value($PackageTxnManager_UpgradeEvent_type_value())
    var $t40: $Value; // $PackageTxnManager_TwoPhaseUpgrade_type_value()
    var $t41: $Value; // $IntegerType()
    var $t42: $Value; // $BooleanType()
    var $t43: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t44: $Value; // $Config_ModifyConfigCapability_type_value($Version_Version_type_value())
    var $t45: $Value; // $Event_EventHandle_type_value($PackageTxnManager_UpgradeEvent_type_value())
    var $t46: $Value; // $BooleanType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,2263,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := strategy;
      assume {:print "$track_local(7,2263,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t16 := move(account)
    call $t16 := $CopyOrMoveValue(account);

    // $t17 := move(strategy)
    call $t17 := $CopyOrMoveValue(strategy);

    // $t18 := 0
    $t18 := $Integer(0);

    // $t19 := ==($t17, $t18)
    $t19 := $Boolean($IsEqual($t17, $t18));

    // if ($t19) goto L0 else goto L1
    if (b#$Boolean($t19)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t20 := true
    $t20 := $Boolean(true);

    // tmp#$7 := $t20
    call tmp#$7 := $CopyOrMoveValue($t20);
    if (true) {
     $trace_temp := tmp#$7;
      assume {:print "$track_local(7,2428,7):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t21 := 1
    $t21 := $Integer(1);

    // tmp#$7 := ==($t17, $t21)
    tmp#$7 := $Boolean($IsEqual($t17, $t21));
    if (true) {
     $trace_temp := tmp#$7;
      assume {:print "$track_local(7,2471,7):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L3:
L3:

    // if (tmp#$7) goto L4 else goto L5
    if (b#$Boolean(tmp#$7)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // goto L6
    goto L6;

    // L4:
L4:

    // $t22 := true
    $t22 := $Boolean(true);

    // tmp#$8 := $t22
    call tmp#$8 := $CopyOrMoveValue($t22);
    if (true) {
     $trace_temp := tmp#$8;
      assume {:print "$track_local(7,2428,8):", $trace_temp} true;
    }

    // goto L7
    goto L7;

    // L6:
L6:

    // $t23 := 2
    $t23 := $Integer(2);

    // tmp#$8 := ==($t17, $t23)
    tmp#$8 := $Boolean($IsEqual($t17, $t23));
    if (true) {
     $trace_temp := tmp#$8;
      assume {:print "$track_local(7,2505,8):", $trace_temp} true;
    }

    // goto L7
    goto L7;

    // L7:
L7:

    // if (tmp#$8) goto L8 else goto L9
    if (b#$Boolean(tmp#$8)) { goto L8; } else { goto L9; }

    // L9:
L9:

    // goto L10
    goto L10;

    // L8:
L8:

    // $t24 := true
    $t24 := $Boolean(true);

    // tmp#$9 := $t24
    call tmp#$9 := $CopyOrMoveValue($t24);
    if (true) {
     $trace_temp := tmp#$9;
      assume {:print "$track_local(7,2428,9):", $trace_temp} true;
    }

    // goto L11
    goto L11;

    // L10:
L10:

    // $t25 := 3
    $t25 := $Integer(3);

    // tmp#$9 := ==($t17, $t25)
    tmp#$9 := $Boolean($IsEqual($t17, $t25));
    if (true) {
     $trace_temp := tmp#$9;
      assume {:print "$track_local(7,2540,9):", $trace_temp} true;
    }

    // goto L11
    goto L11;

    // L11:
L11:

    // $t26 := 108
    $t26 := $Integer(108);

    // $t27 := Errors::invalid_argument($t26)
    call $t27 := $Errors_invalid_argument($t26);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2568):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if (tmp#$9) goto L12 else goto L13
    if (b#$Boolean(tmp#$9)) { goto L12; } else { goto L13; }

    // L13:
L13:

    // destroy($t16)

    // abort($t27)
    $abort_code := i#$Integer($t27);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2421):", $trace_abort_temp} true;
    }
    goto Abort;

    // L12:
L12:

    // account_address := Signer::address_of($t16)
    call account_address := $Signer_address_of($t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2648):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // previous_strategy := PackageTxnManager::get_module_upgrade_strategy(account_address)
    call previous_strategy := $PackageTxnManager_get_module_upgrade_strategy(account_address);
    if ($abort_flag) {
      goto Abort;
    }

    // $t28 := >($t17, previous_strategy)
    call $t28 := $Gt($t17, previous_strategy);

    // $t29 := 106
    $t29 := $Integer(106);

    // $t30 := Errors::invalid_argument($t29)
    call $t30 := $Errors_invalid_argument($t29);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2808):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t28) goto L14 else goto L15
    if (b#$Boolean($t28)) { goto L14; } else { goto L15; }

    // L15:
L15:

    // destroy($t16)

    // abort($t30)
    $abort_code := i#$Integer($t30);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2763):", $trace_abort_temp} true;
    }
    goto Abort;

    // L14:
L14:

    // $t31 := exists<PackageTxnManager::ModuleUpgradeStrategy>(account_address)
    $t31 := $ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, account_address);

    // if ($t31) goto L16 else goto L17
    if (b#$Boolean($t31)) { goto L16; } else { goto L17; }

    // L17:
L17:

    // goto L18
    goto L18;

    // L16:
L16:

    // $t32 := borrow_global<PackageTxnManager::ModuleUpgradeStrategy>(account_address)
    call $t32 := $BorrowGlobal($PackageTxnManager_ModuleUpgradeStrategy_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2930):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref($t32)

    // $t33 := borrow_field<PackageTxnManager::ModuleUpgradeStrategy>.strategy($t32)
    call $t33 := $BorrowField($t32, $PackageTxnManager_ModuleUpgradeStrategy_strategy);

    // unpack_ref($t33)

    // write_ref($t33, $t17)
    call $t33 := $WriteRef($t33, $t17);

    // pack_ref($t33)

    // write_back[Reference($t32)]($t33)
    call $t32 := $WritebackToReference($t33, $t32);

    // pack_ref($t32)

    // write_back[PackageTxnManager::ModuleUpgradeStrategy]($t32)
    call $PackageTxnManager_ModuleUpgradeStrategy_$memory := $WritebackToGlobal($PackageTxnManager_ModuleUpgradeStrategy_$memory, $t32);

    // goto L19
    goto L19;

    // L18:
L18:

    // $t34 := pack PackageTxnManager::ModuleUpgradeStrategy($t17)
    call $t34 := $PackageTxnManager_ModuleUpgradeStrategy_pack(0, 0, 0, $t17);

    // move_to<PackageTxnManager::ModuleUpgradeStrategy>($t34, $t16)
    call $PackageTxnManager_ModuleUpgradeStrategy_$memory := $MoveTo($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $t34, $t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3044):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // goto L19
    goto L19;

    // L19:
L19:

    // $t35 := 1
    $t35 := $Integer(1);

    // $t36 := ==($t17, $t35)
    $t36 := $Boolean($IsEqual($t17, $t35));

    // if ($t36) goto L20 else goto L21
    if (b#$Boolean($t36)) { goto L20; } else { goto L21; }

    // L21:
L21:

    // goto L22
    goto L22;

    // L20:
L20:

    // version_cap := Config::extract_modify_config_capability<Version::Version>($t16)
    call version_cap := $Config_extract_modify_config_capability($Version_Version_type_value(), $t16);
    if ($abort_flag) {
      goto Abort;
    }

    // $t37 := pack PackageTxnManager::UpgradePlanCapability(account_address)
    call $t37 := $PackageTxnManager_UpgradePlanCapability_pack(0, 0, 0, account_address);

    // move_to<PackageTxnManager::UpgradePlanCapability>($t37, $t16)
    call $PackageTxnManager_UpgradePlanCapability_$memory := $MoveTo($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $t37, $t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3289):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t38 := Option::none<PackageTxnManager::UpgradePlan>()
    call $t38 := $Option_none($PackageTxnManager_UpgradePlan_type_value());
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3428):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t39 := Event::new_event_handle<PackageTxnManager::UpgradeEvent>($t16)
    call $t39 := $Event_new_event_handle($PackageTxnManager_UpgradeEvent_type_value(), $t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3537):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t40 := pack PackageTxnManager::TwoPhaseUpgrade($t38, version_cap, $t39)
    call $t40 := $PackageTxnManager_TwoPhaseUpgrade_pack(0, 0, 0, $t38, version_cap, $t39);

    // move_to<PackageTxnManager::TwoPhaseUpgrade>($t40, $t16)
    call $PackageTxnManager_TwoPhaseUpgrade_$memory := $MoveTo($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $t40, $t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3381):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // goto L23
    goto L23;

    // L22:
L22:

    // destroy($t16)

    // goto L23
    goto L23;

    // L23:
L23:

    // $t41 := 1
    $t41 := $Integer(1);

    // $t42 := ==(previous_strategy, $t41)
    $t42 := $Boolean($IsEqual(previous_strategy, $t41));

    // if ($t42) goto L24 else goto L25
    if (b#$Boolean($t42)) { goto L24; } else { goto L25; }

    // L25:
L25:

    // goto L26
    goto L26;

    // L24:
L24:

    // tpu := move_from<PackageTxnManager::TwoPhaseUpgrade>(account_address)
    call $PackageTxnManager_TwoPhaseUpgrade_$memory, tpu := $MoveFrom($PackageTxnManager_TwoPhaseUpgrade_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3732):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := tpu;
      assume {:print "$track_local(7,3732,12):", $trace_temp} true;
    }

    // ($t43, $t44, $t45) := unpack PackageTxnManager::TwoPhaseUpgrade(tpu)
    call $t43, $t44, $t45 := $PackageTxnManager_TwoPhaseUpgrade_unpack(tpu);




    // upgrade_event := $t45
    call upgrade_event := $CopyOrMoveValue($t45);
    if (true) {
     $trace_temp := upgrade_event;
      assume {:print "$track_local(7,3834,13):", $trace_temp} true;
    }

    // version_cap#452 := $t44
    call version_cap#452 := $CopyOrMoveValue($t44);
    if (true) {
     $trace_temp := version_cap#452;
      assume {:print "$track_local(7,3821,15):", $trace_temp} true;
    }

    // destroy($t43)

    // Event::destroy_handle<PackageTxnManager::UpgradeEvent>(upgrade_event)
    call $Event_destroy_handle($PackageTxnManager_UpgradeEvent_type_value(), upgrade_event);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3879):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // Config::destroy_modify_config_capability<Version::Version>(version_cap#452)
    call $Config_destroy_modify_config_capability($Version_Version_type_value(), version_cap#452);
    if ($abort_flag) {
      goto Abort;
    }

    // $t46 := exists<PackageTxnManager::UpgradePlanCapability>(account_address)
    $t46 := $ResourceExists($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, account_address);

    // if ($t46) goto L27 else goto L28
    if (b#$Boolean($t46)) { goto L27; } else { goto L28; }

    // L28:
L28:

    // goto L26
    goto L26;

    // L27:
L27:

    // cap := move_from<PackageTxnManager::UpgradePlanCapability>(account_address)
    call $PackageTxnManager_UpgradePlanCapability_$memory, cap := $MoveFrom($PackageTxnManager_UpgradePlanCapability_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,4176):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,4176,3):", $trace_temp} true;
    }

    // PackageTxnManager::destroy_upgrade_plan_cap(cap)
    call $PackageTxnManager_destroy_upgrade_plan_cap(cap);
    if ($abort_flag) {
      goto Abort;
    }

    // goto L26
    goto L26;

    // L26:
L26:

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:inline 1} $PackageTxnManager_update_module_upgrade_strategy_$direct_inter(account: $Value, strategy: $Value) returns ()
{
    assume is#$Address(account);

    assume $IsValidU8(strategy);

    call $PackageTxnManager_update_module_upgrade_strategy_$def(account, strategy);
}


procedure {:inline 1} $PackageTxnManager_update_module_upgrade_strategy_$direct_intra(account: $Value, strategy: $Value) returns ()
{
    assume is#$Address(account);

    assume $IsValidU8(strategy);

    call $PackageTxnManager_update_module_upgrade_strategy_$def(account, strategy);
}


procedure {:inline 1} $PackageTxnManager_update_module_upgrade_strategy(account: $Value, strategy: $Value) returns ()
{
    assume is#$Address(account);

    assume $IsValidU8(strategy);

    call $PackageTxnManager_update_module_upgrade_strategy_$def(account, strategy);
}


procedure {:inline 1} $PackageTxnManager_update_module_upgrade_strategy_$def_verify(account: $Value, strategy: $Value) returns ()
{
    // declare local variables
    var account_address: $Value; // $AddressType()
    var cap: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var previous_strategy: $Value; // $IntegerType()
    var tmp#$5: $Value; // $BooleanType()
    var tmp#$6: $Value; // $IntegerType()
    var tmp#$7: $Value; // $BooleanType()
    var tmp#$8: $Value; // $BooleanType()
    var tmp#$9: $Value; // $BooleanType()
    var tmp#$10: $Value; // $BooleanType()
    var tmp#$11: $Value; // $IntegerType()
    var tpu: $Value; // $PackageTxnManager_TwoPhaseUpgrade_type_value()
    var upgrade_event: $Value; // $Event_EventHandle_type_value($PackageTxnManager_UpgradeEvent_type_value())
    var version_cap: $Value; // $Config_ModifyConfigCapability_type_value($Version_Version_type_value())
    var version_cap#452: $Value; // $Config_ModifyConfigCapability_type_value($Version_Version_type_value())
    var $t16: $Value; // $AddressType()
    var $t17: $Value; // $IntegerType()
    var $t18: $Value; // $IntegerType()
    var $t19: $Value; // $BooleanType()
    var $t20: $Value; // $BooleanType()
    var $t21: $Value; // $IntegerType()
    var $t22: $Value; // $BooleanType()
    var $t23: $Value; // $IntegerType()
    var $t24: $Value; // $BooleanType()
    var $t25: $Value; // $IntegerType()
    var $t26: $Value; // $IntegerType()
    var $t27: $Value; // $IntegerType()
    var $t28: $Value; // $BooleanType()
    var $t29: $Value; // $IntegerType()
    var $t30: $Value; // $IntegerType()
    var $t31: $Value; // $BooleanType()
    var $t32: $Mutation; // ReferenceType($PackageTxnManager_ModuleUpgradeStrategy_type_value())
    var $t33: $Mutation; // ReferenceType($IntegerType())
    var $t34: $Value; // $PackageTxnManager_ModuleUpgradeStrategy_type_value()
    var $t35: $Value; // $IntegerType()
    var $t36: $Value; // $BooleanType()
    var $t37: $Value; // $PackageTxnManager_UpgradePlanCapability_type_value()
    var $t38: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t39: $Value; // $Event_EventHandle_type_value($PackageTxnManager_UpgradeEvent_type_value())
    var $t40: $Value; // $PackageTxnManager_TwoPhaseUpgrade_type_value()
    var $t41: $Value; // $IntegerType()
    var $t42: $Value; // $BooleanType()
    var $t43: $Value; // $Option_Option_type_value($PackageTxnManager_UpgradePlan_type_value())
    var $t44: $Value; // $Config_ModifyConfigCapability_type_value($Version_Version_type_value())
    var $t45: $Value; // $Event_EventHandle_type_value($PackageTxnManager_UpgradeEvent_type_value())
    var $t46: $Value; // $BooleanType()
    var $trace_temp: $Value;
    var $trace_abort_temp: int;

    // initialize function execution
    assume !$abort_flag;

    // track values of parameters at entry time
    if (true) {
     $trace_temp := account;
      assume {:print "$track_local(7,2263,0):", $trace_temp} true;
    }
    if (true) {
     $trace_temp := strategy;
      assume {:print "$track_local(7,2263,1):", $trace_temp} true;
    }

    // bytecode translation starts here
    // $t16 := move(account)
    call $t16 := $CopyOrMoveValue(account);

    // $t17 := move(strategy)
    call $t17 := $CopyOrMoveValue(strategy);

    // $t18 := 0
    $t18 := $Integer(0);

    // $t19 := ==($t17, $t18)
    $t19 := $Boolean($IsEqual($t17, $t18));

    // if ($t19) goto L0 else goto L1
    if (b#$Boolean($t19)) { goto L0; } else { goto L1; }

    // L1:
L1:

    // goto L2
    goto L2;

    // L0:
L0:

    // $t20 := true
    $t20 := $Boolean(true);

    // tmp#$7 := $t20
    call tmp#$7 := $CopyOrMoveValue($t20);
    if (true) {
     $trace_temp := tmp#$7;
      assume {:print "$track_local(7,2428,7):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L2:
L2:

    // $t21 := 1
    $t21 := $Integer(1);

    // tmp#$7 := ==($t17, $t21)
    tmp#$7 := $Boolean($IsEqual($t17, $t21));
    if (true) {
     $trace_temp := tmp#$7;
      assume {:print "$track_local(7,2471,7):", $trace_temp} true;
    }

    // goto L3
    goto L3;

    // L3:
L3:

    // if (tmp#$7) goto L4 else goto L5
    if (b#$Boolean(tmp#$7)) { goto L4; } else { goto L5; }

    // L5:
L5:

    // goto L6
    goto L6;

    // L4:
L4:

    // $t22 := true
    $t22 := $Boolean(true);

    // tmp#$8 := $t22
    call tmp#$8 := $CopyOrMoveValue($t22);
    if (true) {
     $trace_temp := tmp#$8;
      assume {:print "$track_local(7,2428,8):", $trace_temp} true;
    }

    // goto L7
    goto L7;

    // L6:
L6:

    // $t23 := 2
    $t23 := $Integer(2);

    // tmp#$8 := ==($t17, $t23)
    tmp#$8 := $Boolean($IsEqual($t17, $t23));
    if (true) {
     $trace_temp := tmp#$8;
      assume {:print "$track_local(7,2505,8):", $trace_temp} true;
    }

    // goto L7
    goto L7;

    // L7:
L7:

    // if (tmp#$8) goto L8 else goto L9
    if (b#$Boolean(tmp#$8)) { goto L8; } else { goto L9; }

    // L9:
L9:

    // goto L10
    goto L10;

    // L8:
L8:

    // $t24 := true
    $t24 := $Boolean(true);

    // tmp#$9 := $t24
    call tmp#$9 := $CopyOrMoveValue($t24);
    if (true) {
     $trace_temp := tmp#$9;
      assume {:print "$track_local(7,2428,9):", $trace_temp} true;
    }

    // goto L11
    goto L11;

    // L10:
L10:

    // $t25 := 3
    $t25 := $Integer(3);

    // tmp#$9 := ==($t17, $t25)
    tmp#$9 := $Boolean($IsEqual($t17, $t25));
    if (true) {
     $trace_temp := tmp#$9;
      assume {:print "$track_local(7,2540,9):", $trace_temp} true;
    }

    // goto L11
    goto L11;

    // L11:
L11:

    // $t26 := 108
    $t26 := $Integer(108);

    // $t27 := Errors::invalid_argument($t26)
    call $t27 := $Errors_invalid_argument_$direct_inter($t26);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2568):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if (tmp#$9) goto L12 else goto L13
    if (b#$Boolean(tmp#$9)) { goto L12; } else { goto L13; }

    // L13:
L13:

    // destroy($t16)

    // abort($t27)
    $abort_code := i#$Integer($t27);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2421):", $trace_abort_temp} true;
    }
    goto Abort;

    // L12:
L12:

    // account_address := Signer::address_of($t16)
    call account_address := $Signer_address_of_$direct_inter($t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2648):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // previous_strategy := PackageTxnManager::get_module_upgrade_strategy(account_address)
    call previous_strategy := $PackageTxnManager_get_module_upgrade_strategy_$direct_intra(account_address);
    if ($abort_flag) {
      goto Abort;
    }

    // $t28 := >($t17, previous_strategy)
    call $t28 := $Gt($t17, previous_strategy);

    // $t29 := 106
    $t29 := $Integer(106);

    // $t30 := Errors::invalid_argument($t29)
    call $t30 := $Errors_invalid_argument_$direct_inter($t29);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2808):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // if ($t28) goto L14 else goto L15
    if (b#$Boolean($t28)) { goto L14; } else { goto L15; }

    // L15:
L15:

    // destroy($t16)

    // abort($t30)
    $abort_code := i#$Integer($t30);
    if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2763):", $trace_abort_temp} true;
    }
    goto Abort;

    // L14:
L14:

    // $t31 := exists<PackageTxnManager::ModuleUpgradeStrategy>(account_address)
    $t31 := $ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, account_address);

    // if ($t31) goto L16 else goto L17
    if (b#$Boolean($t31)) { goto L16; } else { goto L17; }

    // L17:
L17:

    // goto L18
    goto L18;

    // L16:
L16:

    // $t32 := borrow_global<PackageTxnManager::ModuleUpgradeStrategy>(account_address)
    call $t32 := $BorrowGlobal($PackageTxnManager_ModuleUpgradeStrategy_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,2930):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // unpack_ref($t32)

    // $t33 := borrow_field<PackageTxnManager::ModuleUpgradeStrategy>.strategy($t32)
    call $t33 := $BorrowField($t32, $PackageTxnManager_ModuleUpgradeStrategy_strategy);

    // unpack_ref($t33)

    // write_ref($t33, $t17)
    call $t33 := $WriteRef($t33, $t17);

    // pack_ref($t33)

    // write_back[Reference($t32)]($t33)
    call $t32 := $WritebackToReference($t33, $t32);

    // pack_ref($t32)

    // write_back[PackageTxnManager::ModuleUpgradeStrategy]($t32)
    call $PackageTxnManager_ModuleUpgradeStrategy_$memory := $WritebackToGlobal($PackageTxnManager_ModuleUpgradeStrategy_$memory, $t32);

    // goto L19
    goto L19;

    // L18:
L18:

    // $t34 := pack PackageTxnManager::ModuleUpgradeStrategy($t17)
    call $t34 := $PackageTxnManager_ModuleUpgradeStrategy_pack(0, 0, 0, $t17);

    // move_to<PackageTxnManager::ModuleUpgradeStrategy>($t34, $t16)
    call $PackageTxnManager_ModuleUpgradeStrategy_$memory := $MoveTo($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $t34, $t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3044):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // goto L19
    goto L19;

    // L19:
L19:

    // $t35 := 1
    $t35 := $Integer(1);

    // $t36 := ==($t17, $t35)
    $t36 := $Boolean($IsEqual($t17, $t35));

    // if ($t36) goto L20 else goto L21
    if (b#$Boolean($t36)) { goto L20; } else { goto L21; }

    // L21:
L21:

    // goto L22
    goto L22;

    // L20:
L20:

    // version_cap := Config::extract_modify_config_capability<Version::Version>($t16)
    call version_cap := $Config_extract_modify_config_capability_$direct_inter($Version_Version_type_value(), $t16);
    if ($abort_flag) {
      goto Abort;
    }

    // $t37 := pack PackageTxnManager::UpgradePlanCapability(account_address)
    call $t37 := $PackageTxnManager_UpgradePlanCapability_pack(0, 0, 0, account_address);

    // move_to<PackageTxnManager::UpgradePlanCapability>($t37, $t16)
    call $PackageTxnManager_UpgradePlanCapability_$memory := $MoveTo($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $t37, $t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3289):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t38 := Option::none<PackageTxnManager::UpgradePlan>()
    call $t38 := $Option_none_$direct_inter($PackageTxnManager_UpgradePlan_type_value());
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3428):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t39 := Event::new_event_handle<PackageTxnManager::UpgradeEvent>($t16)
    call $t39 := $Event_new_event_handle($PackageTxnManager_UpgradeEvent_type_value(), $t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3537):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // $t40 := pack PackageTxnManager::TwoPhaseUpgrade($t38, version_cap, $t39)
    call $t40 := $PackageTxnManager_TwoPhaseUpgrade_pack(0, 0, 0, $t38, version_cap, $t39);

    // move_to<PackageTxnManager::TwoPhaseUpgrade>($t40, $t16)
    call $PackageTxnManager_TwoPhaseUpgrade_$memory := $MoveTo($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $t40, $t16);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3381):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // goto L23
    goto L23;

    // L22:
L22:

    // destroy($t16)

    // goto L23
    goto L23;

    // L23:
L23:

    // $t41 := 1
    $t41 := $Integer(1);

    // $t42 := ==(previous_strategy, $t41)
    $t42 := $Boolean($IsEqual(previous_strategy, $t41));

    // if ($t42) goto L24 else goto L25
    if (b#$Boolean($t42)) { goto L24; } else { goto L25; }

    // L25:
L25:

    // goto L26
    goto L26;

    // L24:
L24:

    // tpu := move_from<PackageTxnManager::TwoPhaseUpgrade>(account_address)
    call $PackageTxnManager_TwoPhaseUpgrade_$memory, tpu := $MoveFrom($PackageTxnManager_TwoPhaseUpgrade_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3732):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := tpu;
      assume {:print "$track_local(7,3732,12):", $trace_temp} true;
    }

    // ($t43, $t44, $t45) := unpack PackageTxnManager::TwoPhaseUpgrade(tpu)
    call $t43, $t44, $t45 := $PackageTxnManager_TwoPhaseUpgrade_unpack(tpu);




    // upgrade_event := $t45
    call upgrade_event := $CopyOrMoveValue($t45);
    if (true) {
     $trace_temp := upgrade_event;
      assume {:print "$track_local(7,3834,13):", $trace_temp} true;
    }

    // version_cap#452 := $t44
    call version_cap#452 := $CopyOrMoveValue($t44);
    if (true) {
     $trace_temp := version_cap#452;
      assume {:print "$track_local(7,3821,15):", $trace_temp} true;
    }

    // destroy($t43)

    // Event::destroy_handle<PackageTxnManager::UpgradeEvent>(upgrade_event)
    call $Event_destroy_handle($PackageTxnManager_UpgradeEvent_type_value(), upgrade_event);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,3879):", $trace_abort_temp} true;
    }
      goto Abort;
    }

    // Config::destroy_modify_config_capability<Version::Version>(version_cap#452)
    call $Config_destroy_modify_config_capability_$direct_inter($Version_Version_type_value(), version_cap#452);
    if ($abort_flag) {
      goto Abort;
    }

    // $t46 := exists<PackageTxnManager::UpgradePlanCapability>(account_address)
    $t46 := $ResourceExists($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, account_address);

    // if ($t46) goto L27 else goto L28
    if (b#$Boolean($t46)) { goto L27; } else { goto L28; }

    // L28:
L28:

    // goto L26
    goto L26;

    // L27:
L27:

    // cap := move_from<PackageTxnManager::UpgradePlanCapability>(account_address)
    call $PackageTxnManager_UpgradePlanCapability_$memory, cap := $MoveFrom($PackageTxnManager_UpgradePlanCapability_$memory, account_address, $EmptyTypeValueArray);
    if ($abort_flag) {
      if (true) {
     $trace_abort_temp := $abort_code;
      assume {:print "$track_abort(7,4176):", $trace_abort_temp} true;
    }
      goto Abort;
    }
    if (true) {
     $trace_temp := cap;
      assume {:print "$track_local(7,4176,3):", $trace_temp} true;
    }

    // PackageTxnManager::destroy_upgrade_plan_cap(cap)
    call $PackageTxnManager_destroy_upgrade_plan_cap_$direct_intra(cap);
    if ($abort_flag) {
      goto Abort;
    }

    // goto L26
    goto L26;

    // L26:
L26:

    // return ()
    return;

Abort:
    $abort_flag := true;
}

procedure {:timeLimit 40} $PackageTxnManager_update_module_upgrade_strategy_$verify(account: $Value, strategy: $Value) returns ()
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean(!$IsEqual(strategy, $Integer(0)))) && b#$Boolean($Boolean(!$IsEqual(strategy, $Integer(1)))))) && b#$Boolean($Boolean(!$IsEqual(strategy, $Integer(2)))))) && b#$Boolean($Boolean(!$IsEqual(strategy, $Integer(3))))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account))) && b#$Boolean($Boolean(i#$Integer(strategy) <= i#$Integer($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_ModuleUpgradeStrategy_strategy))))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account))))) && b#$Boolean($Boolean($IsEqual(strategy, $Integer(0))))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual(strategy, $Integer(1)))) && b#$Boolean($ResourceExists($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual(strategy, $Integer(1)))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($Config_ModifyConfigCapabilityHolder_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Version_Version_type_value()], 1), $Signer_$address_of(account)))))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual(strategy, $Integer(1)))) && b#$Boolean($Option_spec_is_none($Config_ModifyConfigCapability_type_value($Version_Version_type_value()), $SelectField($PackageTxnManager_holder$21($Config_ModifyConfigCapabilityHolder_$memory, account), $Config_ModifyConfigCapabilityHolder_cap)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual(strategy, $Integer(1)))) && b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)))))) ==> $abort_flag;
ensures b#$Boolean(old($Boolean(b#$Boolean($Boolean(b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account))) && b#$Boolean($Boolean($IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1)))))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)))))))) ==> $abort_flag;
ensures $abort_flag ==> (b#$Boolean(old($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean(b#$Boolean($Boolean(!$IsEqual(strategy, $Integer(0)))) && b#$Boolean($Boolean(!$IsEqual(strategy, $Integer(1)))))) && b#$Boolean($Boolean(!$IsEqual(strategy, $Integer(2)))))) && b#$Boolean($Boolean(!$IsEqual(strategy, $Integer(3)))))))
    || b#$Boolean(old($Boolean(b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account))) && b#$Boolean($Boolean(i#$Integer(strategy) <= i#$Integer($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_ModuleUpgradeStrategy_strategy)))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account))))) && b#$Boolean($Boolean($IsEqual(strategy, $Integer(0)))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual(strategy, $Integer(1)))) && b#$Boolean($ResourceExists($PackageTxnManager_UpgradePlanCapability_$memory, $EmptyTypeValueArray, $Signer_$address_of(account))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual(strategy, $Integer(1)))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($Config_ModifyConfigCapabilityHolder_$memory, $TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $Version_Version_type_value()], 1), $Signer_$address_of(account))))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual(strategy, $Integer(1)))) && b#$Boolean($Option_spec_is_none($Config_ModifyConfigCapability_type_value($Version_Version_type_value()), $SelectField($PackageTxnManager_holder$21($Config_ModifyConfigCapabilityHolder_$memory, account), $Config_ModifyConfigCapabilityHolder_cap))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean($IsEqual(strategy, $Integer(1)))) && b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $Signer_$address_of(account))))))
    || b#$Boolean(old($Boolean(b#$Boolean($Boolean(b#$Boolean($ResourceExists($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account))) && b#$Boolean($Boolean($IsEqual($SelectField($ResourceValue($PackageTxnManager_ModuleUpgradeStrategy_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)), $PackageTxnManager_ModuleUpgradeStrategy_strategy), $Integer(1)))))) && b#$Boolean($Boolean(!b#$Boolean($ResourceExists($PackageTxnManager_TwoPhaseUpgrade_$memory, $EmptyTypeValueArray, $Signer_$address_of(account)))))))));
{
    assume is#$Address(account);

    assume $IsValidU8(strategy);

    call $InitVerification();
    assume (forall $inv_addr: int, $inv_tv0: $TypeValue :: {contents#$Memory($Config_ModifyConfigCapabilityHolder_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr]}
        $Config_ModifyConfigCapabilityHolder_$is_well_formed(contents#$Memory($Config_ModifyConfigCapabilityHolder_$memory)[$TypeValueArray($MapConstTypeValue($DefaultTypeValue())[0 := $inv_tv0], 1), $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_ModuleUpgradeStrategy_$is_well_formed(contents#$Memory($PackageTxnManager_ModuleUpgradeStrategy_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_TwoPhaseUpgrade_$is_well_formed(contents#$Memory($PackageTxnManager_TwoPhaseUpgrade_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    assume (forall $inv_addr: int :: {contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr]}
        $PackageTxnManager_UpgradePlanCapability_$is_well_formed(contents#$Memory($PackageTxnManager_UpgradePlanCapability_$memory)[$EmptyTypeValueArray, $inv_addr])
    );
    call $PackageTxnManager_update_module_upgrade_strategy_$def_verify(account, strategy);
}
