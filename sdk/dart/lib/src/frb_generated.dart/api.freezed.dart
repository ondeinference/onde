// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'api.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$OndeError {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OndeError);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'OndeError()';
}


}

/// @nodoc
class $OndeErrorCopyWith<$Res>  {
$OndeErrorCopyWith(OndeError _, $Res Function(OndeError) __);
}


/// Adds pattern-matching-related methods to [OndeError].
extension OndeErrorPatterns on OndeError {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( OndeError_NoModelLoaded value)?  noModelLoaded,TResult Function( OndeError_AlreadyLoaded value)?  alreadyLoaded,TResult Function( OndeError_ModelBuild value)?  modelBuild,TResult Function( OndeError_Inference value)?  inference,TResult Function( OndeError_Cancelled value)?  cancelled,TResult Function( OndeError_Other value)?  other,required TResult orElse(),}){
final _that = this;
switch (_that) {
case OndeError_NoModelLoaded() when noModelLoaded != null:
return noModelLoaded(_that);case OndeError_AlreadyLoaded() when alreadyLoaded != null:
return alreadyLoaded(_that);case OndeError_ModelBuild() when modelBuild != null:
return modelBuild(_that);case OndeError_Inference() when inference != null:
return inference(_that);case OndeError_Cancelled() when cancelled != null:
return cancelled(_that);case OndeError_Other() when other != null:
return other(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( OndeError_NoModelLoaded value)  noModelLoaded,required TResult Function( OndeError_AlreadyLoaded value)  alreadyLoaded,required TResult Function( OndeError_ModelBuild value)  modelBuild,required TResult Function( OndeError_Inference value)  inference,required TResult Function( OndeError_Cancelled value)  cancelled,required TResult Function( OndeError_Other value)  other,}){
final _that = this;
switch (_that) {
case OndeError_NoModelLoaded():
return noModelLoaded(_that);case OndeError_AlreadyLoaded():
return alreadyLoaded(_that);case OndeError_ModelBuild():
return modelBuild(_that);case OndeError_Inference():
return inference(_that);case OndeError_Cancelled():
return cancelled(_that);case OndeError_Other():
return other(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( OndeError_NoModelLoaded value)?  noModelLoaded,TResult? Function( OndeError_AlreadyLoaded value)?  alreadyLoaded,TResult? Function( OndeError_ModelBuild value)?  modelBuild,TResult? Function( OndeError_Inference value)?  inference,TResult? Function( OndeError_Cancelled value)?  cancelled,TResult? Function( OndeError_Other value)?  other,}){
final _that = this;
switch (_that) {
case OndeError_NoModelLoaded() when noModelLoaded != null:
return noModelLoaded(_that);case OndeError_AlreadyLoaded() when alreadyLoaded != null:
return alreadyLoaded(_that);case OndeError_ModelBuild() when modelBuild != null:
return modelBuild(_that);case OndeError_Inference() when inference != null:
return inference(_that);case OndeError_Cancelled() when cancelled != null:
return cancelled(_that);case OndeError_Other() when other != null:
return other(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  noModelLoaded,TResult Function( String modelName)?  alreadyLoaded,TResult Function( String reason)?  modelBuild,TResult Function( String reason)?  inference,TResult Function()?  cancelled,TResult Function( String reason)?  other,required TResult orElse(),}) {final _that = this;
switch (_that) {
case OndeError_NoModelLoaded() when noModelLoaded != null:
return noModelLoaded();case OndeError_AlreadyLoaded() when alreadyLoaded != null:
return alreadyLoaded(_that.modelName);case OndeError_ModelBuild() when modelBuild != null:
return modelBuild(_that.reason);case OndeError_Inference() when inference != null:
return inference(_that.reason);case OndeError_Cancelled() when cancelled != null:
return cancelled();case OndeError_Other() when other != null:
return other(_that.reason);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  noModelLoaded,required TResult Function( String modelName)  alreadyLoaded,required TResult Function( String reason)  modelBuild,required TResult Function( String reason)  inference,required TResult Function()  cancelled,required TResult Function( String reason)  other,}) {final _that = this;
switch (_that) {
case OndeError_NoModelLoaded():
return noModelLoaded();case OndeError_AlreadyLoaded():
return alreadyLoaded(_that.modelName);case OndeError_ModelBuild():
return modelBuild(_that.reason);case OndeError_Inference():
return inference(_that.reason);case OndeError_Cancelled():
return cancelled();case OndeError_Other():
return other(_that.reason);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  noModelLoaded,TResult? Function( String modelName)?  alreadyLoaded,TResult? Function( String reason)?  modelBuild,TResult? Function( String reason)?  inference,TResult? Function()?  cancelled,TResult? Function( String reason)?  other,}) {final _that = this;
switch (_that) {
case OndeError_NoModelLoaded() when noModelLoaded != null:
return noModelLoaded();case OndeError_AlreadyLoaded() when alreadyLoaded != null:
return alreadyLoaded(_that.modelName);case OndeError_ModelBuild() when modelBuild != null:
return modelBuild(_that.reason);case OndeError_Inference() when inference != null:
return inference(_that.reason);case OndeError_Cancelled() when cancelled != null:
return cancelled();case OndeError_Other() when other != null:
return other(_that.reason);case _:
  return null;

}
}

}

/// @nodoc


class OndeError_NoModelLoaded extends OndeError {
  const OndeError_NoModelLoaded(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OndeError_NoModelLoaded);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'OndeError.noModelLoaded()';
}


}




/// @nodoc


class OndeError_AlreadyLoaded extends OndeError {
  const OndeError_AlreadyLoaded({required this.modelName}): super._();
  

 final  String modelName;

/// Create a copy of OndeError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OndeError_AlreadyLoadedCopyWith<OndeError_AlreadyLoaded> get copyWith => _$OndeError_AlreadyLoadedCopyWithImpl<OndeError_AlreadyLoaded>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OndeError_AlreadyLoaded&&(identical(other.modelName, modelName) || other.modelName == modelName));
}


@override
int get hashCode => Object.hash(runtimeType,modelName);

@override
String toString() {
  return 'OndeError.alreadyLoaded(modelName: $modelName)';
}


}

/// @nodoc
abstract mixin class $OndeError_AlreadyLoadedCopyWith<$Res> implements $OndeErrorCopyWith<$Res> {
  factory $OndeError_AlreadyLoadedCopyWith(OndeError_AlreadyLoaded value, $Res Function(OndeError_AlreadyLoaded) _then) = _$OndeError_AlreadyLoadedCopyWithImpl;
@useResult
$Res call({
 String modelName
});




}
/// @nodoc
class _$OndeError_AlreadyLoadedCopyWithImpl<$Res>
    implements $OndeError_AlreadyLoadedCopyWith<$Res> {
  _$OndeError_AlreadyLoadedCopyWithImpl(this._self, this._then);

  final OndeError_AlreadyLoaded _self;
  final $Res Function(OndeError_AlreadyLoaded) _then;

/// Create a copy of OndeError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? modelName = null,}) {
  return _then(OndeError_AlreadyLoaded(
modelName: null == modelName ? _self.modelName : modelName // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class OndeError_ModelBuild extends OndeError {
  const OndeError_ModelBuild({required this.reason}): super._();
  

 final  String reason;

/// Create a copy of OndeError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OndeError_ModelBuildCopyWith<OndeError_ModelBuild> get copyWith => _$OndeError_ModelBuildCopyWithImpl<OndeError_ModelBuild>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OndeError_ModelBuild&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'OndeError.modelBuild(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $OndeError_ModelBuildCopyWith<$Res> implements $OndeErrorCopyWith<$Res> {
  factory $OndeError_ModelBuildCopyWith(OndeError_ModelBuild value, $Res Function(OndeError_ModelBuild) _then) = _$OndeError_ModelBuildCopyWithImpl;
@useResult
$Res call({
 String reason
});




}
/// @nodoc
class _$OndeError_ModelBuildCopyWithImpl<$Res>
    implements $OndeError_ModelBuildCopyWith<$Res> {
  _$OndeError_ModelBuildCopyWithImpl(this._self, this._then);

  final OndeError_ModelBuild _self;
  final $Res Function(OndeError_ModelBuild) _then;

/// Create a copy of OndeError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,}) {
  return _then(OndeError_ModelBuild(
reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class OndeError_Inference extends OndeError {
  const OndeError_Inference({required this.reason}): super._();
  

 final  String reason;

/// Create a copy of OndeError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OndeError_InferenceCopyWith<OndeError_Inference> get copyWith => _$OndeError_InferenceCopyWithImpl<OndeError_Inference>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OndeError_Inference&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'OndeError.inference(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $OndeError_InferenceCopyWith<$Res> implements $OndeErrorCopyWith<$Res> {
  factory $OndeError_InferenceCopyWith(OndeError_Inference value, $Res Function(OndeError_Inference) _then) = _$OndeError_InferenceCopyWithImpl;
@useResult
$Res call({
 String reason
});




}
/// @nodoc
class _$OndeError_InferenceCopyWithImpl<$Res>
    implements $OndeError_InferenceCopyWith<$Res> {
  _$OndeError_InferenceCopyWithImpl(this._self, this._then);

  final OndeError_Inference _self;
  final $Res Function(OndeError_Inference) _then;

/// Create a copy of OndeError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,}) {
  return _then(OndeError_Inference(
reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class OndeError_Cancelled extends OndeError {
  const OndeError_Cancelled(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OndeError_Cancelled);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'OndeError.cancelled()';
}


}




/// @nodoc


class OndeError_Other extends OndeError {
  const OndeError_Other({required this.reason}): super._();
  

 final  String reason;

/// Create a copy of OndeError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OndeError_OtherCopyWith<OndeError_Other> get copyWith => _$OndeError_OtherCopyWithImpl<OndeError_Other>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OndeError_Other&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'OndeError.other(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $OndeError_OtherCopyWith<$Res> implements $OndeErrorCopyWith<$Res> {
  factory $OndeError_OtherCopyWith(OndeError_Other value, $Res Function(OndeError_Other) _then) = _$OndeError_OtherCopyWithImpl;
@useResult
$Res call({
 String reason
});




}
/// @nodoc
class _$OndeError_OtherCopyWithImpl<$Res>
    implements $OndeError_OtherCopyWith<$Res> {
  _$OndeError_OtherCopyWithImpl(this._self, this._then);

  final OndeError_Other _self;
  final $Res Function(OndeError_Other) _then;

/// Create a copy of OndeError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,}) {
  return _then(OndeError_Other(
reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
