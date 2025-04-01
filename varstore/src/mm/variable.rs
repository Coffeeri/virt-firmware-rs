use virtfw_libefi::efivar::types::EfiVarAttr;

pub enum MmVariableFunction {
    GetVariable = 1,
    GetNextVariableName = 2,
    SetVariable = 3,
    QueryVariableInfo = 4,
    ReadyToBoot = 5,
    ExitBootService = 6,
    LockVariable = 8,
    GetPayloadSize = 11,
}

// SMM_VARIABLE_COMMUNICATE_HEADER
pub struct MmVariableHeader {
    pub function: u64,
    pub status: u64,
}

// SMM_VARIABLE_COMMUNICATE_ACCESS_VARIABLE
pub struct MmVariableAccess {
    pub guid: [u8; 16],
    pub data_size: u64,
    pub name_size: u64,
    pub attributes: EfiVarAttr,
    // Name
    // Data
}

// SMM_VARIABLE_COMMUNICATE_GET_NEXT_VARIABLE_NAME
pub struct MmNextVariable {
    pub guid: [u8; 16],
    pub name_size: u64,
    // Name
}

// SMM_VARIABLE_COMMUNICATE_QUERY_VARIABLE_INFO
pub struct MmVariableInfo {
    pub max_storage_size: u64,
    pub free_storage_size: u64,
    pub max_variable_size: u64,
    pub attributes: u32,
}

// SMM_VARIABLE_COMMUNICATE_GET_PAYLOAD_SIZE
pub struct MmGetPayloadSize {
    pub payload_size: u64,
}
