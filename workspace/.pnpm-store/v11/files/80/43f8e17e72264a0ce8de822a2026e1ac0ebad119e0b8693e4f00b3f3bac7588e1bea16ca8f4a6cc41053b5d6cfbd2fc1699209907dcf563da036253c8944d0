// src/codes.ts
var CODAMA_ERROR__UNRECOGNIZED_NODE_KIND = 1;
var CODAMA_ERROR__UNEXPECTED_NODE_KIND = 2;
var CODAMA_ERROR__UNEXPECTED_NESTED_NODE_KIND = 3;
var CODAMA_ERROR__LINKED_NODE_NOT_FOUND = 4;
var CODAMA_ERROR__NODE_FILESYSTEM_FUNCTION_UNAVAILABLE = 5;
var CODAMA_ERROR__VERSION_MISMATCH = 6;
var CODAMA_ERROR__UNRECOGNIZED_NUMBER_FORMAT = 7;
var CODAMA_ERROR__UNRECOGNIZED_BYTES_ENCODING = 8;
var CODAMA_ERROR__ENUM_VARIANT_NOT_FOUND = 9;
var CODAMA_ERROR__DISCRIMINATOR_FIELD_NOT_FOUND = 10;
var CODAMA_ERROR__DISCRIMINATOR_FIELD_HAS_NO_DEFAULT_VALUE = 11;
var CODAMA_ERROR__VISITORS__CANNOT_ADD_DUPLICATED_PDA_NAMES = 12e5;
var CODAMA_ERROR__VISITORS__INVALID_PDA_SEED_VALUES = 1200001;
var CODAMA_ERROR__VISITORS__CYCLIC_DEPENDENCY_DETECTED_WHEN_RESOLVING_INSTRUCTION_DEFAULT_VALUES = 1200002;
var CODAMA_ERROR__VISITORS__CANNOT_USE_OPTIONAL_ACCOUNT_AS_PDA_SEED_VALUE = 1200003;
var CODAMA_ERROR__VISITORS__INVALID_INSTRUCTION_DEFAULT_VALUE_DEPENDENCY = 1200004;
var CODAMA_ERROR__VISITORS__ACCOUNT_FIELD_NOT_FOUND = 1200005;
var CODAMA_ERROR__VISITORS__INVALID_NUMBER_WRAPPER = 1200006;
var CODAMA_ERROR__VISITORS__CANNOT_EXTEND_MISSING_VISIT_FUNCTION = 1200007;
var CODAMA_ERROR__VISITORS__FAILED_TO_VALIDATE_NODE = 1200008;
var CODAMA_ERROR__VISITORS__INSTRUCTION_ENUM_ARGUMENT_NOT_FOUND = 1200009;
var CODAMA_ERROR__VISITORS__CANNOT_FLATTEN_STRUCT_WITH_CONFLICTING_ATTRIBUTES = 1200010;
var CODAMA_ERROR__VISITORS__RENDER_MAP_KEY_NOT_FOUND = 1200011;
var CODAMA_ERROR__VISITORS__CANNOT_REMOVE_LAST_PATH_IN_NODE_STACK = 1200012;
var CODAMA_ERROR__ANCHOR__UNRECOGNIZED_IDL_TYPE = 21e5;
var CODAMA_ERROR__ANCHOR__ACCOUNT_TYPE_MISSING = 2100001;
var CODAMA_ERROR__ANCHOR__ARGUMENT_TYPE_MISSING = 2100002;
var CODAMA_ERROR__ANCHOR__TYPE_PATH_MISSING = 2100003;
var CODAMA_ERROR__ANCHOR__SEED_KIND_UNIMPLEMENTED = 2100004;
var CODAMA_ERROR__ANCHOR__PROGRAM_ID_KIND_UNIMPLEMENTED = 2100005;
var CODAMA_ERROR__ANCHOR__GENERIC_TYPE_MISSING = 2100006;
var CODAMA_ERROR__ANCHOR__EVENT_TYPE_MISSING = 2100007;
var CODAMA_ERROR__DYNAMIC_CLIENT__INSTRUCTION_NOT_FOUND = 25e5;
var CODAMA_ERROR__DYNAMIC_CLIENT__PDA_NOT_FOUND = 2500001;
var CODAMA_ERROR__DYNAMIC_CLIENT__NODE_REFERENCE_NOT_FOUND = 2500002;
var CODAMA_ERROR__DYNAMIC_CLIENT__ACCOUNT_MISSING = 2500003;
var CODAMA_ERROR__DYNAMIC_CLIENT__INVALID_ACCOUNT_ADDRESS = 2500004;
var CODAMA_ERROR__DYNAMIC_CLIENT__UNEXPECTED_ADDRESS_TYPE = 2500005;
var CODAMA_ERROR__DYNAMIC_CLIENT__CANNOT_CONVERT_TO_ADDRESS = 2500006;
var CODAMA_ERROR__DYNAMIC_CLIENT__ACCOUNT_RESOLVER_MISSING = 2500007;
var CODAMA_ERROR__DYNAMIC_CLIENT__CIRCULAR_ACCOUNT_DEPENDENCY = 2500008;
var CODAMA_ERROR__DYNAMIC_CLIENT__UNSUPPORTED_OPTIONAL_ACCOUNT_STRATEGY = 2500009;
var CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_DERIVE_PDA = 2500010;
var CODAMA_ERROR__DYNAMIC_CLIENT__ARGUMENT_MISSING = 2500011;
var CODAMA_ERROR__DYNAMIC_CLIENT__DEFAULT_VALUE_MISSING = 2500012;
var CODAMA_ERROR__DYNAMIC_CLIENT__INVALID_ARGUMENT_INPUT = 2500013;
var CODAMA_ERROR__DYNAMIC_CLIENT__UNEXPECTED_ARGUMENT_TYPE = 2500014;
var CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_ENCODE_ARGUMENT = 2500015;
var CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_EXECUTE_RESOLVER = 2500016;
var CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_VALIDATE_INPUT = 2500017;
var CODAMA_ERROR__DYNAMIC_CLIENT__UNSUPPORTED_NODE = 2500018;
var CODAMA_ERROR__DYNAMIC_CLIENT__INVARIANT_VIOLATION = 2500019;
var CODAMA_ERROR__RENDERERS__UNSUPPORTED_NODE = 28e5;
var CODAMA_ERROR__RENDERERS__MISSING_DEPENDENCY_VERSIONS = 2800001;

// src/context.ts
function encodeValue(value) {
  if (Array.isArray(value)) {
    const commaSeparatedValues = value.map(encodeValue).join(
      "%2C%20"
      /* ", " */
    );
    return "%5B" + commaSeparatedValues + /* "]" */
    "%5D";
  } else if (typeof value === "bigint") {
    return `${value}n`;
  } else {
    return encodeURIComponent(
      String(
        value != null && Object.getPrototypeOf(value) === null ? (
          // Plain objects with no protoype don't have a `toString` method.
          // Convert them before stringifying them.
          { ...value }
        ) : value
      )
    );
  }
}
function encodeObjectContextEntry([key, value]) {
  return `${key}=${encodeValue(value)}`;
}
function encodeContextObject(context) {
  const searchParamsString = Object.entries(context).map(encodeObjectContextEntry).join("&");
  return btoa(searchParamsString);
}

// src/messages.ts
var CodamaErrorMessages = {
  [CODAMA_ERROR__ANCHOR__ACCOUNT_TYPE_MISSING]: "Account type [$name] is missing from the IDL types.",
  [CODAMA_ERROR__ANCHOR__ARGUMENT_TYPE_MISSING]: "Argument name [$name] is missing from the instruction definition.",
  [CODAMA_ERROR__ANCHOR__EVENT_TYPE_MISSING]: "Event type [$name] is missing from the IDL types.",
  [CODAMA_ERROR__ANCHOR__GENERIC_TYPE_MISSING]: "Generic type [$name] is missing from the IDL types.",
  [CODAMA_ERROR__ANCHOR__PROGRAM_ID_KIND_UNIMPLEMENTED]: "Program ID kind [$kind] is not implemented.",
  [CODAMA_ERROR__ANCHOR__SEED_KIND_UNIMPLEMENTED]: "Seed kind [$kind] is not implemented.",
  [CODAMA_ERROR__ANCHOR__TYPE_PATH_MISSING]: "Field type is missing for path [$path] in [$idlType].",
  [CODAMA_ERROR__ANCHOR__UNRECOGNIZED_IDL_TYPE]: "Unrecognized Anchor IDL type [$idlType].",
  [CODAMA_ERROR__DISCRIMINATOR_FIELD_HAS_NO_DEFAULT_VALUE]: "Discriminator field [$field] has no default value.",
  [CODAMA_ERROR__DISCRIMINATOR_FIELD_NOT_FOUND]: "Could not find discriminator field [$field]",
  [CODAMA_ERROR__DYNAMIC_CLIENT__ACCOUNT_MISSING]: "Missing account [$accountName] in [$instructionName] instruction.",
  [CODAMA_ERROR__DYNAMIC_CLIENT__ACCOUNT_RESOLVER_MISSING]: "Resolver [$resolverName] not provided for account [$accountName].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__ARGUMENT_MISSING]: "Missing argument [$argumentName] in [$instructionName].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__CANNOT_CONVERT_TO_ADDRESS]: "Cannot convert value to Address: [$value].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__CIRCULAR_ACCOUNT_DEPENDENCY]: "Circular dependency detected: [$chain].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__DEFAULT_VALUE_MISSING]: "Default value is missing for argument [$argumentName] in [$instructionName].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_DERIVE_PDA]: "Failed to derive PDA for account [$accountName].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_ENCODE_ARGUMENT]: "Failed to encode argument [$argumentName] in [$instructionName].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_EXECUTE_RESOLVER]: "Resolver [$resolverName] threw an error while resolving [$targetKind] [$targetName].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_VALIDATE_INPUT]: "Failed to validate input: [$message].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__INSTRUCTION_NOT_FOUND]: "Instruction [$instructionName] not found in IDL. Available: [$availableIxs].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__INVALID_ACCOUNT_ADDRESS]: "Invalid account address [$accountName]: [$value].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__INVALID_ARGUMENT_INPUT]: "Invalid argument input [$argumentName]: [$value]. Expected [$expectedType].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__INVARIANT_VIOLATION]: "Internal invariant violation: [$message].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__NODE_REFERENCE_NOT_FOUND]: "Referenced node [$referencedName] not found in [$instructionName].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__PDA_NOT_FOUND]: "PDA [$pdaName] not found in IDL. Available: [$available].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__UNEXPECTED_ADDRESS_TYPE]: "Expected [$expectedType] for account [$accountName], but received [$actualType].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__UNEXPECTED_ARGUMENT_TYPE]: "Expected [$expectedType] for [$nodeKind], but received [$actualType].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__UNSUPPORTED_NODE]: "Unsupported node kind [$nodeKind].",
  [CODAMA_ERROR__DYNAMIC_CLIENT__UNSUPPORTED_OPTIONAL_ACCOUNT_STRATEGY]: "Unsupported optional account strategy [$strategy] for account [$accountName] in [$instructionName].",
  [CODAMA_ERROR__ENUM_VARIANT_NOT_FOUND]: "Enum variant [$variant] not found in enum type [$enumName].",
  [CODAMA_ERROR__LINKED_NODE_NOT_FOUND]: "Could not find linked node [$name] from [$kind].",
  [CODAMA_ERROR__NODE_FILESYSTEM_FUNCTION_UNAVAILABLE]: "Node.js filesystem function [$fsFunction] is not available in your environment.",
  [CODAMA_ERROR__RENDERERS__MISSING_DEPENDENCY_VERSIONS]: "No version specified for the following dependencies: [$dependencies]. $message",
  [CODAMA_ERROR__RENDERERS__UNSUPPORTED_NODE]: "Cannot render the encountered node of kind [$kind].",
  [CODAMA_ERROR__UNEXPECTED_NESTED_NODE_KIND]: "Expected nested node of kind [$expectedKinds], got [$kind]",
  [CODAMA_ERROR__UNEXPECTED_NODE_KIND]: "Expected node of kind [$expectedKinds], got [$kind].",
  [CODAMA_ERROR__UNRECOGNIZED_BYTES_ENCODING]: "Unrecognized bytes encoding [$encoding].",
  [CODAMA_ERROR__UNRECOGNIZED_NODE_KIND]: "Unrecognized node kind [$kind].",
  [CODAMA_ERROR__UNRECOGNIZED_NUMBER_FORMAT]: "Unrecognized number format [$format].",
  [CODAMA_ERROR__VERSION_MISMATCH]: "The provided RootNode version [$rootVersion] is not compatible with the installed Codama version [$codamaVersion].",
  [CODAMA_ERROR__VISITORS__ACCOUNT_FIELD_NOT_FOUND]: "Account [$name] does not have a field named [$missingField].",
  [CODAMA_ERROR__VISITORS__CANNOT_ADD_DUPLICATED_PDA_NAMES]: "Cannot add PDAs to program [$programName] because the following PDA names already exist [$duplicatedPdaNames].",
  [CODAMA_ERROR__VISITORS__CANNOT_EXTEND_MISSING_VISIT_FUNCTION]: "Cannot extend visitor with function [$visitFunction] as the base visitor does not support it.",
  [CODAMA_ERROR__VISITORS__CANNOT_FLATTEN_STRUCT_WITH_CONFLICTING_ATTRIBUTES]: "Cannot flatten struct since this would cause the following attributes to conflict [$conflictingAttributes].",
  [CODAMA_ERROR__VISITORS__CANNOT_REMOVE_LAST_PATH_IN_NODE_STACK]: "Cannot remove the last path in the node stack.",
  [CODAMA_ERROR__VISITORS__CANNOT_USE_OPTIONAL_ACCOUNT_AS_PDA_SEED_VALUE]: "Cannot use optional account [$seedValueName] as the [$seedName] PDA seed for the [$instructionAccountName] account of the [$instructionName] instruction.",
  [CODAMA_ERROR__VISITORS__CYCLIC_DEPENDENCY_DETECTED_WHEN_RESOLVING_INSTRUCTION_DEFAULT_VALUES]: "Circular dependency detected when resolving the accounts and arguments' default values of the [$instructionName] instruction. Got the following dependency cycle [$formattedCycle].",
  [CODAMA_ERROR__VISITORS__FAILED_TO_VALIDATE_NODE]: "Failed to validate the given node [$formattedHistogram].",
  [CODAMA_ERROR__VISITORS__INSTRUCTION_ENUM_ARGUMENT_NOT_FOUND]: "Could not find an enum argument named [$argumentName] for instruction [$instructionName].",
  [CODAMA_ERROR__VISITORS__INVALID_INSTRUCTION_DEFAULT_VALUE_DEPENDENCY]: "Dependency [$dependencyName] of kind [$dependencyKind] is not a valid dependency of [$parentName] of kind [$parentKind] in the [$instructionName] instruction.",
  [CODAMA_ERROR__VISITORS__INVALID_NUMBER_WRAPPER]: "Invalid number wrapper kind [$wrapper].",
  [CODAMA_ERROR__VISITORS__INVALID_PDA_SEED_VALUES]: "Invalid seed values for PDA [$pdaName] in instruction [$instructionName].",
  [CODAMA_ERROR__VISITORS__RENDER_MAP_KEY_NOT_FOUND]: "Cannot find key [$key] in RenderMap."
};

// src/message-formatter.ts
function getHumanReadableErrorMessage(code, context = {}) {
  const messageFormatString = CodamaErrorMessages[code];
  const message = messageFormatString.replace(
    /(?<!\\)\$(\w+)/g,
    (substring, variableName) => variableName in context ? `${context[variableName]}` : substring
  );
  return message;
}
function getErrorMessage(code, context = {}) {
  if (process.env.NODE_ENV !== "production") {
    return getHumanReadableErrorMessage(code, context);
  } else {
    let decodingAdviceMessage = `Codama error #${code}; Decode this error by running \`npx @codama/errors decode -- ${code}`;
    if (Object.keys(context).length) {
      decodingAdviceMessage += ` '${encodeContextObject(context)}'`;
    }
    return `${decodingAdviceMessage}\``;
  }
}

// src/error.ts
function isCodamaError(e, code) {
  const isCodamaError2 = e instanceof Error && e.name === "CodamaError";
  if (isCodamaError2) {
    if (code !== void 0) {
      return e.context.__code === code;
    }
    return true;
  }
  return false;
}
var CodamaError = class extends Error {
  context;
  constructor(...[code, contextAndErrorOptions]) {
    let context;
    let errorOptions;
    if (contextAndErrorOptions) {
      const { cause, ...contextRest } = contextAndErrorOptions;
      if (cause) {
        errorOptions = { cause };
      }
      if (Object.keys(contextRest).length > 0) {
        context = contextRest;
      }
    }
    const message = getErrorMessage(code, context);
    super(message, errorOptions);
    this.context = {
      __code: code,
      ...context
    };
    this.name = "CodamaError";
  }
};

// src/logs.ts
function logError(message) {
  console.error(message);
}
function logWarn(message) {
  console.warn(message);
}
function logInfo(message) {
  console.log(message);
}

// src/stack-trace.ts
function safeCaptureStackTrace(...args) {
  if ("captureStackTrace" in Error && typeof Error.captureStackTrace === "function") {
    Error.captureStackTrace(...args);
  }
}

export { CODAMA_ERROR__ANCHOR__ACCOUNT_TYPE_MISSING, CODAMA_ERROR__ANCHOR__ARGUMENT_TYPE_MISSING, CODAMA_ERROR__ANCHOR__EVENT_TYPE_MISSING, CODAMA_ERROR__ANCHOR__GENERIC_TYPE_MISSING, CODAMA_ERROR__ANCHOR__PROGRAM_ID_KIND_UNIMPLEMENTED, CODAMA_ERROR__ANCHOR__SEED_KIND_UNIMPLEMENTED, CODAMA_ERROR__ANCHOR__TYPE_PATH_MISSING, CODAMA_ERROR__ANCHOR__UNRECOGNIZED_IDL_TYPE, CODAMA_ERROR__DISCRIMINATOR_FIELD_HAS_NO_DEFAULT_VALUE, CODAMA_ERROR__DISCRIMINATOR_FIELD_NOT_FOUND, CODAMA_ERROR__DYNAMIC_CLIENT__ACCOUNT_MISSING, CODAMA_ERROR__DYNAMIC_CLIENT__ACCOUNT_RESOLVER_MISSING, CODAMA_ERROR__DYNAMIC_CLIENT__ARGUMENT_MISSING, CODAMA_ERROR__DYNAMIC_CLIENT__CANNOT_CONVERT_TO_ADDRESS, CODAMA_ERROR__DYNAMIC_CLIENT__CIRCULAR_ACCOUNT_DEPENDENCY, CODAMA_ERROR__DYNAMIC_CLIENT__DEFAULT_VALUE_MISSING, CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_DERIVE_PDA, CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_ENCODE_ARGUMENT, CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_EXECUTE_RESOLVER, CODAMA_ERROR__DYNAMIC_CLIENT__FAILED_TO_VALIDATE_INPUT, CODAMA_ERROR__DYNAMIC_CLIENT__INSTRUCTION_NOT_FOUND, CODAMA_ERROR__DYNAMIC_CLIENT__INVALID_ACCOUNT_ADDRESS, CODAMA_ERROR__DYNAMIC_CLIENT__INVALID_ARGUMENT_INPUT, CODAMA_ERROR__DYNAMIC_CLIENT__INVARIANT_VIOLATION, CODAMA_ERROR__DYNAMIC_CLIENT__NODE_REFERENCE_NOT_FOUND, CODAMA_ERROR__DYNAMIC_CLIENT__PDA_NOT_FOUND, CODAMA_ERROR__DYNAMIC_CLIENT__UNEXPECTED_ADDRESS_TYPE, CODAMA_ERROR__DYNAMIC_CLIENT__UNEXPECTED_ARGUMENT_TYPE, CODAMA_ERROR__DYNAMIC_CLIENT__UNSUPPORTED_NODE, CODAMA_ERROR__DYNAMIC_CLIENT__UNSUPPORTED_OPTIONAL_ACCOUNT_STRATEGY, CODAMA_ERROR__ENUM_VARIANT_NOT_FOUND, CODAMA_ERROR__LINKED_NODE_NOT_FOUND, CODAMA_ERROR__NODE_FILESYSTEM_FUNCTION_UNAVAILABLE, CODAMA_ERROR__RENDERERS__MISSING_DEPENDENCY_VERSIONS, CODAMA_ERROR__RENDERERS__UNSUPPORTED_NODE, CODAMA_ERROR__UNEXPECTED_NESTED_NODE_KIND, CODAMA_ERROR__UNEXPECTED_NODE_KIND, CODAMA_ERROR__UNRECOGNIZED_BYTES_ENCODING, CODAMA_ERROR__UNRECOGNIZED_NODE_KIND, CODAMA_ERROR__UNRECOGNIZED_NUMBER_FORMAT, CODAMA_ERROR__VERSION_MISMATCH, CODAMA_ERROR__VISITORS__ACCOUNT_FIELD_NOT_FOUND, CODAMA_ERROR__VISITORS__CANNOT_ADD_DUPLICATED_PDA_NAMES, CODAMA_ERROR__VISITORS__CANNOT_EXTEND_MISSING_VISIT_FUNCTION, CODAMA_ERROR__VISITORS__CANNOT_FLATTEN_STRUCT_WITH_CONFLICTING_ATTRIBUTES, CODAMA_ERROR__VISITORS__CANNOT_REMOVE_LAST_PATH_IN_NODE_STACK, CODAMA_ERROR__VISITORS__CANNOT_USE_OPTIONAL_ACCOUNT_AS_PDA_SEED_VALUE, CODAMA_ERROR__VISITORS__CYCLIC_DEPENDENCY_DETECTED_WHEN_RESOLVING_INSTRUCTION_DEFAULT_VALUES, CODAMA_ERROR__VISITORS__FAILED_TO_VALIDATE_NODE, CODAMA_ERROR__VISITORS__INSTRUCTION_ENUM_ARGUMENT_NOT_FOUND, CODAMA_ERROR__VISITORS__INVALID_INSTRUCTION_DEFAULT_VALUE_DEPENDENCY, CODAMA_ERROR__VISITORS__INVALID_NUMBER_WRAPPER, CODAMA_ERROR__VISITORS__INVALID_PDA_SEED_VALUES, CODAMA_ERROR__VISITORS__RENDER_MAP_KEY_NOT_FOUND, CodamaError, isCodamaError, logError, logInfo, logWarn, safeCaptureStackTrace };
//# sourceMappingURL=index.react-native.mjs.map
//# sourceMappingURL=index.react-native.mjs.map