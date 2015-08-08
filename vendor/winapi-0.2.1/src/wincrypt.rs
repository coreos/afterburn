// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
//! Cryptographic API Prototypes and Definitions
//108
pub const ALG_CLASS_ANY: ALG_ID = 0;
pub const ALG_CLASS_SIGNATURE: ALG_ID = 1 << 13;
pub const ALG_CLASS_MSG_ENCRYPT: ALG_ID = 2 << 13;
pub const ALG_CLASS_DATA_ENCRYPT: ALG_ID = 3 << 13;
pub const ALG_CLASS_HASH: ALG_ID = 4 << 13;
pub const ALG_CLASS_KEY_EXCHANGE: ALG_ID = 5 << 13;
pub const ALG_CLASS_ALL: ALG_ID = 7 << 13;
pub const ALG_TYPE_ANY: ALG_ID = 0;
pub const ALG_TYPE_DSS: ALG_ID = 1 << 9;
pub const ALG_TYPE_RSA: ALG_ID = 2 << 9;
pub const ALG_TYPE_BLOCK: ALG_ID = 3 << 9;
pub const ALG_TYPE_STREAM: ALG_ID = 4 << 9;
pub const ALG_TYPE_DH: ALG_ID = 5 << 9;
pub const ALG_TYPE_SECURECHANNEL: ALG_ID = 6 << 9;
pub const ALG_SID_ANY: ALG_ID = 0;
pub const ALG_SID_RSA_ANY: ALG_ID = 0;
pub const ALG_SID_RSA_PKCS: ALG_ID = 1;
pub const ALG_SID_RSA_MSATWORK: ALG_ID = 2;
pub const ALG_SID_RSA_ENTRUST: ALG_ID = 3;
pub const ALG_SID_RSA_PGP: ALG_ID = 4;
pub const ALG_SID_DSS_ANY: ALG_ID = 0;
pub const ALG_SID_DSS_PKCS: ALG_ID = 1;
pub const ALG_SID_DSS_DMS: ALG_ID = 2;
pub const ALG_SID_ECDSA: ALG_ID = 3;
pub const ALG_SID_DES: ALG_ID = 1;
pub const ALG_SID_3DES: ALG_ID = 3;
pub const ALG_SID_DESX: ALG_ID = 4;
pub const ALG_SID_IDEA: ALG_ID = 5;
pub const ALG_SID_CAST: ALG_ID = 6;
pub const ALG_SID_SAFERSK64: ALG_ID = 7;
pub const ALG_SID_SAFERSK128: ALG_ID = 8;
pub const ALG_SID_3DES_112: ALG_ID = 9;
pub const ALG_SID_CYLINK_MEK: ALG_ID = 12;
pub const ALG_SID_RC5: ALG_ID = 13;
pub const ALG_SID_AES_128: ALG_ID = 14;
pub const ALG_SID_AES_192: ALG_ID = 15;
pub const ALG_SID_AES_256: ALG_ID = 16;
pub const ALG_SID_AES: ALG_ID = 17;
pub const ALG_SID_SKIPJACK: ALG_ID = 10;
pub const ALG_SID_TEK: ALG_ID = 11;
pub const CRYPT_MODE_CBCI: ALG_ID = 6;
pub const CRYPT_MODE_CFBP: ALG_ID = 7;
pub const CRYPT_MODE_OFBP: ALG_ID = 8;
pub const CRYPT_MODE_CBCOFM: ALG_ID = 9;
pub const CRYPT_MODE_CBCOFMI: ALG_ID = 10;
pub const ALG_SID_RC2: ALG_ID = 2;
pub const ALG_SID_RC4: ALG_ID = 1;
pub const ALG_SID_SEAL: ALG_ID = 2;
pub const ALG_SID_DH_SANDF: ALG_ID = 1;
pub const ALG_SID_DH_EPHEM: ALG_ID = 2;
pub const ALG_SID_AGREED_KEY_ANY: ALG_ID = 3;
pub const ALG_SID_KEA: ALG_ID = 4;
pub const ALG_SID_ECDH: ALG_ID = 5;
pub const ALG_SID_MD2: ALG_ID = 1;
pub const ALG_SID_MD4: ALG_ID = 2;
pub const ALG_SID_MD5: ALG_ID = 3;
pub const ALG_SID_SHA: ALG_ID = 4;
pub const ALG_SID_SHA1: ALG_ID = 4;
pub const ALG_SID_MAC: ALG_ID = 5;
pub const ALG_SID_RIPEMD: ALG_ID = 6;
pub const ALG_SID_RIPEMD160: ALG_ID = 7;
pub const ALG_SID_SSL3SHAMD5: ALG_ID = 8;
pub const ALG_SID_HMAC: ALG_ID = 9;
pub const ALG_SID_TLS1PRF: ALG_ID = 10;
pub const ALG_SID_HASH_REPLACE_OWF: ALG_ID = 11;
pub const ALG_SID_SHA_256: ALG_ID = 12;
pub const ALG_SID_SHA_384: ALG_ID = 13;
pub const ALG_SID_SHA_512: ALG_ID = 14;
pub const ALG_SID_SSL3_MASTER: ALG_ID = 1;
pub const ALG_SID_SCHANNEL_MASTER_HASH: ALG_ID = 2;
pub const ALG_SID_SCHANNEL_MAC_KEY: ALG_ID = 3;
pub const ALG_SID_PCT1_MASTER: ALG_ID = 4;
pub const ALG_SID_SSL2_MASTER: ALG_ID = 5;
pub const ALG_SID_TLS1_MASTER: ALG_ID = 6;
pub const ALG_SID_SCHANNEL_ENC_KEY: ALG_ID = 7;
pub const ALG_SID_ECMQV: ALG_ID = 1;
pub const ALG_SID_EXAMPLE: ALG_ID = 80;
pub type ALG_ID = ::c_uint;
pub const CALG_MD2: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_MD2;
pub const CALG_MD4: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_MD4;
pub const CALG_MD5: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_MD5;
pub const CALG_SHA: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_SHA;
pub const CALG_SHA1: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_SHA1;
pub const CALG_MAC: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_MAC;
pub const CALG_RSA_SIGN: ALG_ID = ALG_CLASS_SIGNATURE | ALG_TYPE_RSA | ALG_SID_RSA_ANY;
pub const CALG_DSS_SIGN: ALG_ID = ALG_CLASS_SIGNATURE | ALG_TYPE_DSS | ALG_SID_DSS_ANY;
pub const CALG_NO_SIGN: ALG_ID = ALG_CLASS_SIGNATURE | ALG_TYPE_ANY | ALG_SID_ANY;
pub const CALG_RSA_KEYX: ALG_ID = ALG_CLASS_KEY_EXCHANGE | ALG_TYPE_RSA | ALG_SID_RSA_ANY;
pub const CALG_DES: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_DES;
pub const CALG_3DES_112: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_3DES_112;
pub const CALG_3DES: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_3DES;
pub const CALG_DESX: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_DESX;
pub const CALG_RC2: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_RC2;
pub const CALG_RC4: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_STREAM | ALG_SID_RC4;
pub const CALG_SEAL: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_STREAM | ALG_SID_SEAL;
pub const CALG_DH_SF: ALG_ID = ALG_CLASS_KEY_EXCHANGE | ALG_TYPE_DH | ALG_SID_DH_SANDF;
pub const CALG_DH_EPHEM: ALG_ID = ALG_CLASS_KEY_EXCHANGE | ALG_TYPE_DH | ALG_SID_DH_EPHEM;
pub const CALG_AGREEDKEY_ANY: ALG_ID = ALG_CLASS_KEY_EXCHANGE | ALG_TYPE_DH
    | ALG_SID_AGREED_KEY_ANY;
pub const CALG_KEA_KEYX: ALG_ID = ALG_CLASS_KEY_EXCHANGE | ALG_TYPE_DH | ALG_SID_KEA;
pub const CALG_HUGHES_MD5: ALG_ID = ALG_CLASS_KEY_EXCHANGE | ALG_TYPE_ANY | ALG_SID_MD5;
pub const CALG_SKIPJACK: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_SKIPJACK;
pub const CALG_TEK: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_TEK;
pub const CALG_CYLINK_MEK: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_CYLINK_MEK;
pub const CALG_SSL3_SHAMD5: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_SSL3SHAMD5;
pub const CALG_SSL3_MASTER: ALG_ID = ALG_CLASS_MSG_ENCRYPT | ALG_TYPE_SECURECHANNEL
    | ALG_SID_SSL3_MASTER;
pub const CALG_SCHANNEL_MASTER_HASH: ALG_ID = ALG_CLASS_MSG_ENCRYPT | ALG_TYPE_SECURECHANNEL
    | ALG_SID_SCHANNEL_MASTER_HASH;
pub const CALG_SCHANNEL_MAC_KEY: ALG_ID = ALG_CLASS_MSG_ENCRYPT | ALG_TYPE_SECURECHANNEL
    | ALG_SID_SCHANNEL_MAC_KEY;
pub const CALG_SCHANNEL_ENC_KEY: ALG_ID = ALG_CLASS_MSG_ENCRYPT | ALG_TYPE_SECURECHANNEL
    | ALG_SID_SCHANNEL_ENC_KEY;
pub const CALG_PCT1_MASTER: ALG_ID = ALG_CLASS_MSG_ENCRYPT | ALG_TYPE_SECURECHANNEL
    | ALG_SID_PCT1_MASTER;
pub const CALG_SSL2_MASTER: ALG_ID = ALG_CLASS_MSG_ENCRYPT | ALG_TYPE_SECURECHANNEL
    | ALG_SID_SSL2_MASTER;
pub const CALG_TLS1_MASTER: ALG_ID = ALG_CLASS_MSG_ENCRYPT | ALG_TYPE_SECURECHANNEL
    | ALG_SID_TLS1_MASTER;
pub const CALG_RC5: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_RC5;
pub const CALG_HMAC: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_HMAC;
pub const CALG_TLS1PRF: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_TLS1PRF;
pub const CALG_HASH_REPLACE_OWF: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_HASH_REPLACE_OWF;
pub const CALG_AES_128: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_AES_128;
pub const CALG_AES_192: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_AES_192;
pub const CALG_AES_256: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_AES_256;
pub const CALG_AES: ALG_ID = ALG_CLASS_DATA_ENCRYPT | ALG_TYPE_BLOCK | ALG_SID_AES;
pub const CALG_SHA_256: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_SHA_256;
pub const CALG_SHA_384: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_SHA_384;
pub const CALG_SHA_512: ALG_ID = ALG_CLASS_HASH | ALG_TYPE_ANY | ALG_SID_SHA_512;
pub const CALG_ECDH: ALG_ID = ALG_CLASS_KEY_EXCHANGE | ALG_TYPE_DH | ALG_SID_ECDH;
pub const CALG_ECMQV: ALG_ID = ALG_CLASS_KEY_EXCHANGE | ALG_TYPE_ANY | ALG_SID_ECMQV;
pub const CALG_ECDSA: ALG_ID = ALG_CLASS_SIGNATURE | ALG_TYPE_DSS | ALG_SID_ECDSA;
pub type HCRYPTPROV = ::ULONG_PTR;
pub type HCRYPTKEY = ::ULONG_PTR;
pub type HCRYPTHASH = ::ULONG_PTR;
pub const CRYPT_VERIFYCONTEXT: ::DWORD = 0xF0000000;
pub const CRYPT_NEWKEYSET: ::DWORD = 0x00000008;
pub const CRYPT_DELETEKEYSET: ::DWORD = 0x00000010;
pub const CRYPT_MACHINE_KEYSET: ::DWORD = 0x00000020;
pub const CRYPT_SILENT: ::DWORD = 0x00000040;
pub const CRYPT_DEFAULT_CONTAINER_OPTIONAL: ::DWORD = 0x00000080;
pub const CRYPT_EXPORTABLE: ::DWORD = 0x00000001;
pub const CRYPT_USER_PROTECTED: ::DWORD = 0x00000002;
pub const CRYPT_CREATE_SALT: ::DWORD = 0x00000004;
pub const CRYPT_UPDATE_KEY: ::DWORD = 0x00000008;
pub const CRYPT_NO_SALT: ::DWORD = 0x00000010;
pub const CRYPT_PREGEN: ::DWORD = 0x00000040;
pub const CRYPT_RECIPIENT: ::DWORD = 0x00000010;
pub const CRYPT_INITIATOR: ::DWORD = 0x00000040;
pub const CRYPT_ONLINE: ::DWORD = 0x00000080;
pub const CRYPT_SF: ::DWORD = 0x00000100;
pub const CRYPT_CREATE_IV: ::DWORD = 0x00000200;
pub const CRYPT_KEK: ::DWORD = 0x00000400;
pub const CRYPT_DATA_KEY: ::DWORD = 0x00000800;
pub const CRYPT_VOLATILE: ::DWORD = 0x00001000;
pub const CRYPT_SGCKEY: ::DWORD = 0x00002000;
pub const CRYPT_USER_PROTECTED_STRONG: ::DWORD = 0x00100000;
pub const CRYPT_ARCHIVABLE: ::DWORD = 0x00004000;
pub const CRYPT_FORCE_KEY_PROTECTION_HIGH: ::DWORD = 0x00008000;
pub const RSA1024BIT_KEY: ::DWORD = 0x04000000;
pub const CRYPT_SERVER: ::DWORD = 0x00000400;
pub const KEY_LENGTH_MASK: ::DWORD = 0xFFFF0000;
pub const CRYPT_Y_ONLY: ::DWORD = 0x00000001;
pub const CRYPT_SSL2_FALLBACK: ::DWORD = 0x00000002;
pub const CRYPT_DESTROYKEY: ::DWORD = 0x00000004;
pub const CRYPT_OAEP: ::DWORD = 0x00000040;
pub const CRYPT_BLOB_VER3: ::DWORD = 0x00000080;
pub const CRYPT_IPSEC_HMAC_KEY: ::DWORD = 0x00000100;
pub const CRYPT_DECRYPT_RSA_NO_PADDING_CHECK: ::DWORD = 0x00000020;
pub const CRYPT_SECRETDIGEST: ::DWORD = 0x00000001;
pub const CRYPT_OWF_REPL_LM_HASH: ::DWORD = 0x00000001;
pub const CRYPT_LITTLE_ENDIAN: ::DWORD = 0x00000001;
pub const CRYPT_NOHASHOID: ::DWORD = 0x00000001;
pub const CRYPT_TYPE2_FORMAT: ::DWORD = 0x00000002;
pub const CRYPT_X931_FORMAT: ::DWORD = 0x00000004;
pub const CRYPT_MACHINE_DEFAULT: ::DWORD = 0x00000001;
pub const CRYPT_USER_DEFAULT: ::DWORD = 0x00000002;
pub const CRYPT_DELETE_DEFAULT: ::DWORD = 0x00000004;
pub const SIMPLEBLOB: ::DWORD = 0x1;
pub const PUBLICKEYBLOB: ::DWORD = 0x6;
pub const PRIVATEKEYBLOB: ::DWORD = 0x7;
pub const PLAINTEXTKEYBLOB: ::DWORD = 0x8;
pub const OPAQUEKEYBLOB: ::DWORD = 0x9;
pub const PUBLICKEYBLOBEX: ::DWORD = 0xA;
pub const SYMMETRICWRAPKEYBLOB: ::DWORD = 0xB;
pub const KEYSTATEBLOB: ::DWORD = 0xC;
pub const AT_KEYEXCHANGE: ::DWORD = 1;
pub const AT_SIGNATURE: ::DWORD = 2;
pub const CRYPT_USERDATA: ::DWORD = 1;
pub const KP_IV: ::DWORD = 1;
pub const KP_SALT: ::DWORD = 2;
pub const KP_PADDING: ::DWORD = 3;
pub const KP_MODE: ::DWORD = 4;
pub const KP_MODE_BITS: ::DWORD = 5;
pub const KP_PERMISSIONS: ::DWORD = 6;
pub const KP_ALGID: ::DWORD = 7;
pub const KP_BLOCKLEN: ::DWORD = 8;
pub const KP_KEYLEN: ::DWORD = 9;
pub const KP_SALT_EX: ::DWORD = 10;
pub const KP_P: ::DWORD = 11;
pub const KP_G: ::DWORD = 12;
pub const KP_Q: ::DWORD = 13;
pub const KP_X: ::DWORD = 14;
pub const KP_Y: ::DWORD = 15;
pub const KP_RA: ::DWORD = 16;
pub const KP_RB: ::DWORD = 17;
pub const KP_INFO: ::DWORD = 18;
pub const KP_EFFECTIVE_KEYLEN: ::DWORD = 19;
pub const KP_SCHANNEL_ALG: ::DWORD = 20;
pub const KP_CLIENT_RANDOM: ::DWORD = 21;
pub const KP_SERVER_RANDOM: ::DWORD = 22;
pub const KP_RP: ::DWORD = 23;
pub const KP_PRECOMP_MD5: ::DWORD = 24;
pub const KP_PRECOMP_SHA: ::DWORD = 25;
pub const KP_CERTIFICATE: ::DWORD = 26;
pub const KP_CLEAR_KEY: ::DWORD = 27;
pub const KP_PUB_EX_LEN: ::DWORD = 28;
pub const KP_PUB_EX_VAL: ::DWORD = 29;
pub const KP_KEYVAL: ::DWORD = 30;
pub const KP_ADMIN_PIN: ::DWORD = 31;
pub const KP_KEYEXCHANGE_PIN: ::DWORD = 32;
pub const KP_SIGNATURE_PIN: ::DWORD = 33;
pub const KP_PREHASH: ::DWORD = 34;
pub const KP_ROUNDS: ::DWORD = 35;
pub const KP_OAEP_PARAMS: ::DWORD = 36;
pub const KP_CMS_KEY_INFO: ::DWORD = 37;
pub const KP_CMS_DH_KEY_INFO: ::DWORD = 38;
pub const KP_PUB_PARAMS: ::DWORD = 39;
pub const KP_VERIFY_PARAMS: ::DWORD = 40;
pub const KP_HIGHEST_VERSION: ::DWORD = 41;
pub const KP_GET_USE_COUNT: ::DWORD = 42;
pub const KP_PIN_ID: ::DWORD = 43;
pub const KP_PIN_INFO: ::DWORD = 44;
pub const PKCS5_PADDING: ::DWORD = 1;
pub const RANDOM_PADDING: ::DWORD = 2;
pub const ZERO_PADDING: ::DWORD = 3;
pub const CRYPT_MODE_CBC: ::DWORD = 1;
pub const CRYPT_MODE_ECB: ::DWORD = 2;
pub const CRYPT_MODE_OFB: ::DWORD = 3;
pub const CRYPT_MODE_CFB: ::DWORD = 4;
pub const CRYPT_MODE_CTS: ::DWORD = 5;
pub const CRYPT_ENCRYPT: ::DWORD = 0x0001;
pub const CRYPT_DECRYPT: ::DWORD = 0x0002;
pub const CRYPT_EXPORT: ::DWORD = 0x0004;
pub const CRYPT_READ: ::DWORD = 0x0008;
pub const CRYPT_WRITE: ::DWORD = 0x0010;
pub const CRYPT_MAC: ::DWORD = 0x0020;
pub const CRYPT_EXPORT_KEY: ::DWORD = 0x0040;
pub const CRYPT_IMPORT_KEY: ::DWORD = 0x0080;
pub const CRYPT_ARCHIVE: ::DWORD = 0x0100;
pub const HP_ALGID: ::DWORD = 0x0001;
pub const HP_HASHVAL: ::DWORD = 0x0002;
pub const HP_HASHSIZE: ::DWORD = 0x0004;
pub const HP_HMAC_INFO: ::DWORD = 0x0005;
pub const HP_TLS1PRF_LABEL: ::DWORD = 0x0006;
pub const HP_TLS1PRF_SEED: ::DWORD = 0x0007;
pub const CRYPT_FAILED: ::BOOL = ::FALSE;
pub const CRYPT_SUCCEED: ::BOOL = ::TRUE;
pub const PP_ENUMALGS: ::DWORD = 1;
pub const PP_ENUMCONTAINERS: ::DWORD = 2;
pub const PP_IMPTYPE: ::DWORD = 3;
pub const PP_NAME: ::DWORD = 4;
pub const PP_VERSION: ::DWORD = 5;
pub const PP_CONTAINER: ::DWORD = 6;
pub const PP_CHANGE_PASSWORD: ::DWORD = 7;
pub const PP_KEYSET_SEC_DESCR: ::DWORD = 8;
pub const PP_CERTCHAIN: ::DWORD = 9;
pub const PP_KEY_TYPE_SUBTYPE: ::DWORD = 10;
pub const PP_PROVTYPE: ::DWORD = 16;
pub const PP_KEYSTORAGE: ::DWORD = 17;
pub const PP_APPLI_CERT: ::DWORD = 18;
pub const PP_SYM_KEYSIZE: ::DWORD = 19;
pub const PP_SESSION_KEYSIZE: ::DWORD = 20;
pub const PP_UI_PROMPT: ::DWORD = 21;
pub const PP_ENUMALGS_EX: ::DWORD = 22;
pub const PP_ENUMMANDROOTS: ::DWORD = 25;
pub const PP_ENUMELECTROOTS: ::DWORD = 26;
pub const PP_KEYSET_TYPE: ::DWORD = 27;
pub const PP_ADMIN_PIN: ::DWORD = 31;
pub const PP_KEYEXCHANGE_PIN: ::DWORD = 32;
pub const PP_SIGNATURE_PIN: ::DWORD = 33;
pub const PP_SIG_KEYSIZE_INC: ::DWORD = 34;
pub const PP_KEYX_KEYSIZE_INC: ::DWORD = 35;
pub const PP_UNIQUE_CONTAINER: ::DWORD = 36;
pub const PP_SGC_INFO: ::DWORD = 37;
pub const PP_USE_HARDWARE_RNG: ::DWORD = 38;
pub const PP_KEYSPEC: ::DWORD = 39;
pub const PP_ENUMEX_SIGNING_PROT: ::DWORD = 40;
pub const PP_CRYPT_COUNT_KEY_USE: ::DWORD = 41;
pub const PP_USER_CERTSTORE: ::DWORD = 42;
pub const PP_SMARTCARD_READER: ::DWORD = 43;
pub const PP_SMARTCARD_GUID: ::DWORD = 45;
pub const PP_ROOT_CERTSTORE: ::DWORD = 46;
pub const PP_SMARTCARD_READER_ICON: ::DWORD = 47;
pub const CRYPT_FIRST: ::DWORD = 1;
pub const CRYPT_NEXT: ::DWORD = 2;
pub const CRYPT_SGC_ENUM: ::DWORD = 4;
pub const CRYPT_IMPL_HARDWARE: ::DWORD = 1;
pub const CRYPT_IMPL_SOFTWARE: ::DWORD = 2;
pub const CRYPT_IMPL_MIXED: ::DWORD = 3;
pub const CRYPT_IMPL_UNKNOWN: ::DWORD = 4;
pub const CRYPT_IMPL_REMOVABLE: ::DWORD = 8;
pub const CRYPT_SEC_DESCR: ::DWORD = 0x00000001;
pub const CRYPT_PSTORE: ::DWORD = 0x00000002;
pub const CRYPT_UI_PROMPT: ::DWORD = 0x00000004;
pub const CRYPT_FLAG_PCT1: ::DWORD = 0x0001;
pub const CRYPT_FLAG_SSL2: ::DWORD = 0x0002;
pub const CRYPT_FLAG_SSL3: ::DWORD = 0x0004;
pub const CRYPT_FLAG_TLS1: ::DWORD = 0x0008;
pub const CRYPT_FLAG_IPSEC: ::DWORD = 0x0010;
pub const CRYPT_FLAG_SIGNING: ::DWORD = 0x0020;
pub const CRYPT_SGC: ::DWORD = 0x0001;
pub const CRYPT_FASTSGC: ::DWORD = 0x0002;
pub const PP_CLIENT_HWND: ::DWORD = 1;
pub const PP_CONTEXT_INFO: ::DWORD = 11;
pub const PP_KEYEXCHANGE_KEYSIZE: ::DWORD = 12;
pub const PP_SIGNATURE_KEYSIZE: ::DWORD = 13;
pub const PP_KEYEXCHANGE_ALG: ::DWORD = 14;
pub const PP_SIGNATURE_ALG: ::DWORD = 15;
pub const PP_DELETEKEY: ::DWORD = 24;
pub const PP_PIN_PROMPT_STRING: ::DWORD = 44;
pub const PP_SECURE_KEYEXCHANGE_PIN: ::DWORD = 47;
pub const PP_SECURE_SIGNATURE_PIN: ::DWORD = 48;
pub const PROV_RSA_FULL: ::DWORD = 1;
pub const PROV_RSA_SIG: ::DWORD = 2;
pub const PROV_DSS: ::DWORD = 3;
pub const PROV_FORTEZZA: ::DWORD = 4;
pub const PROV_MS_EXCHANGE: ::DWORD = 5;
pub const PROV_SSL: ::DWORD = 6;
pub const PROV_RSA_SCHANNEL: ::DWORD = 12;
pub const PROV_DSS_DH: ::DWORD = 13;
pub const PROV_EC_ECDSA_SIG: ::DWORD = 14;
pub const PROV_EC_ECNRA_SIG: ::DWORD = 15;
pub const PROV_EC_ECDSA_FULL: ::DWORD = 16;
pub const PROV_EC_ECNRA_FULL: ::DWORD = 17;
pub const PROV_DH_SCHANNEL: ::DWORD = 18;
pub const PROV_SPYRUS_LYNKS: ::DWORD = 20;
pub const PROV_RNG: ::DWORD = 21;
pub const PROV_INTEL_SEC: ::DWORD = 22;
pub const PROV_REPLACE_OWF: ::DWORD = 23;
pub const PROV_RSA_AES: ::DWORD = 24;
pub const MS_DEF_PROV: &'static str = "Microsoft Base Cryptographic Provider v1.0";
pub const MS_ENHANCED_PROV: &'static str = "Microsoft Enhanced Cryptographic Provider v1.0";
pub const MS_STRONG_PROV: &'static str = "Microsoft Strong Cryptographic Provider";
pub const MS_DEF_RSA_SIG_PROV: &'static str = "Microsoft RSA Signature Cryptographic Provider";
pub const MS_DEF_RSA_SCHANNEL_PROV: &'static str = "Microsoft RSA SChannel Cryptographic Provider";
pub const MS_DEF_DSS_PROV: &'static str = "Microsoft Base DSS Cryptographic Provider";
pub const MS_DEF_DSS_DH_PROV: &'static str =
    "Microsoft Base DSS and Diffie-Hellman Cryptographic Provider";
pub const MS_ENH_DSS_DH_PROV: &'static str =
    "Microsoft Enhanced DSS and Diffie-Hellman Cryptographic Provider";
pub const MS_DEF_DH_SCHANNEL_PROV: &'static str = "Microsoft DH SChannel Cryptographic Provider";
pub const MS_SCARD_PROV: &'static str = "Microsoft Base Smart Card Crypto Provider";
pub const MS_ENH_RSA_AES_PROV: &'static str =
    "Microsoft Enhanced RSA and AES Cryptographic Provider";
pub const MS_ENH_RSA_AES_PROV_XP: &'static str =
    "Microsoft Enhanced RSA and AES Cryptographic Provider (Prototype)";
pub const MAXUIDLEN: usize = 64;
pub const EXPO_OFFLOAD_REG_VALUE: &'static str = "ExpoOffload";
pub const EXPO_OFFLOAD_FUNC_NAME: &'static str = "OffloadModExpo";
pub const szKEY_CRYPTOAPI_PRIVATE_KEY_OPTIONS: &'static str =
    "Software\\Policies\\Microsoft\\Cryptography";
pub const szKEY_CACHE_ENABLED: &'static str = "CachePrivateKeys";
pub const szKEY_CACHE_SECONDS: &'static str = "PrivateKeyLifetimeSeconds";
pub const szPRIV_KEY_CACHE_MAX_ITEMS: &'static str = "PrivKeyCacheMaxItems";
pub const cPRIV_KEY_CACHE_MAX_ITEMS_DEFAULT: ::DWORD = 20;
pub const szPRIV_KEY_CACHE_PURGE_INTERVAL_SECONDS: &'static str =
    "PrivKeyCachePurgeIntervalSeconds";
pub const cPRIV_KEY_CACHE_PURGE_INTERVAL_SECONDS_DEFAULT: ::DWORD = 86400;
pub const CUR_BLOB_VERSION: ::DWORD = 2;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CMS_KEY_INFO {
    pub dwVersion: ::DWORD,
    pub Algid: ALG_ID,
    pub pbOID: *mut ::BYTE,
    pub cbOID: ::DWORD,
}
pub type PCMS_KEY_INFO = *mut CMS_KEY_INFO;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct HMAC_INFO {
    pub HashAlgid: ALG_ID,
    pub pbInnerString: *mut ::BYTE,
    pub cbInnerString: ::DWORD,
    pub pbOuterString: *mut ::BYTE,
    pub cbOuterString: ::DWORD,
}
pub type PHMAC_INFO = *mut HMAC_INFO;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct SCHANNEL_ALG {
    pub dwUse: ::DWORD,
    pub Algid: ALG_ID,
    pub cBits: ::DWORD,
    pub dwFlags: ::DWORD,
    pub dwReserved: ::DWORD,
}
pub type PSCHANNEL_ALG = *mut SCHANNEL_ALG;
pub const SCHANNEL_MAC_KEY: ::DWORD = 0x00000000;
pub const SCHANNEL_ENC_KEY: ::DWORD = 0x00000001;
pub const INTERNATIONAL_USAGE: ::DWORD = 0x00000001;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct PROV_ENUMALGS {
    pub aiAlgid: ALG_ID,
    pub dwBitLen: ::DWORD,
    pub dwNameLen: ::DWORD,
    pub szName: [::CHAR; 20],
}
#[repr(C)] #[derive(Copy)]
pub struct PROV_ENUMALGS_EX {
    pub aiAlgid: ALG_ID,
    pub dwDefaultLen: ::DWORD,
    pub dwMinLen: ::DWORD,
    pub dwMaxLen: ::DWORD,
    pub dwProtocols: ::DWORD,
    pub dwNameLen: ::DWORD,
    pub szName: [::CHAR; 20],
    pub dwLongNameLen: ::DWORD,
    pub szLongName: [::CHAR; 40],
}
impl Clone for PROV_ENUMALGS_EX { fn clone(&self) -> PROV_ENUMALGS_EX { *self } }
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct BLOBHEADER {
    pub bType: ::BYTE,
    pub bVersion: ::BYTE,
    pub reserved: ::WORD,
    pub aiKeyAlg: ::ALG_ID,
}
pub type PUBLICKEYSTRUC = BLOBHEADER;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct RSAPUBKEY {
    pub magic: ::DWORD,
    pub bitlen: ::DWORD,
    pub pubexp: ::DWORD,
}
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct DHPUBKEY {
    pub magic: ::DWORD,
    pub bitlen: ::DWORD,
}
pub type DSSPUBKEY = DHPUBKEY;
pub type KEAPUBKEY = DHPUBKEY;
pub type TEKPUBKEY = DHPUBKEY;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct DSSSEED {
    pub counter: ::DWORD,
    pub seed: [::BYTE; 20],
}
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct DHPUBKEY_VER3 {
    pub magic: ::DWORD,
    pub bitlenP: ::DWORD,
    pub bitlenQ: ::DWORD,
    pub bitlenJ: ::DWORD,
    pub DSSSeed: DSSSEED,
}
pub type DSSPUBKEY_VER3 = DHPUBKEY_VER3;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct DHPRIVKEY_VER3 {
    pub magic: ::DWORD,
    pub bitlenP: ::DWORD,
    pub bitlenQ: ::DWORD,
    pub bitlenJ: ::DWORD,
    pub bitlenX: ::DWORD,
    pub DSSSeed: DSSSEED,
}
pub type DSSPRIVKEY_VER3 = DHPRIVKEY_VER3;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct KEY_TYPE_SUBTYPE {
    pub dwKeySpec: ::DWORD,
    pub Type: ::GUID,
    pub Subtype: ::GUID,
}
pub type PKEY_TYPE_SUBTYPE = *mut KEY_TYPE_SUBTYPE;
#[repr(C)] #[derive(Copy)]
pub struct CERT_FORTEZZA_DATA_PROP {
    pub SerialNumber: [::c_uchar; 8],
    pub CertIndex: ::c_int,
    pub CertLabel: [::c_uchar; 36],
}
impl Clone for CERT_FORTEZZA_DATA_PROP { fn clone(&self) -> CERT_FORTEZZA_DATA_PROP { *self } }
#[repr(C)] #[derive(Copy)]
pub struct CRYPT_RC4_KEY_STATE {
    pub Key: [::c_uchar; 16],
    pub SBox: [::c_uchar; 256],
    pub i: ::c_uchar,
    pub j: ::c_uchar,
}
impl Clone for CRYPT_RC4_KEY_STATE { fn clone(&self) -> CRYPT_RC4_KEY_STATE { *self } }
pub type PCRYPT_RC4_KEY_STATE = *mut CRYPT_RC4_KEY_STATE;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_DES_KEY_STATE {
    pub Key: [::c_uchar; 8],
    pub IV: [::c_uchar; 8],
    pub Feedback: [::c_uchar; 8],
}
pub type PCRYPT_DES_KEY_STATE = *mut CRYPT_DES_KEY_STATE;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_3DES_KEY_STATE {
    pub Key: [::c_uchar; 24],
    pub IV: [::c_uchar; 8],
    pub Feedback: [::c_uchar; 8],
}
pub type PCRYPT_3DES_KEY_STATE = *mut CRYPT_3DES_KEY_STATE;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_AES_128_KEY_STATE {
    pub Key: [::c_uchar; 16],
    pub IV: [::c_uchar; 16],
    pub EncryptionState: [[::c_uchar; 16]; 11],
    pub DecryptionState: [[::c_uchar; 16]; 11],
    pub Feedback: [::c_uchar; 16],
}
pub type PCRYPT_AES_128_KEY_STATE = *mut CRYPT_AES_128_KEY_STATE;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_AES_256_KEY_STATE {
    pub Key: [::c_uchar; 32],
    pub IV: [::c_uchar; 16],
    pub EncryptionState: [[::c_uchar; 16]; 15],
    pub DecryptionState: [[::c_uchar; 16]; 15],
    pub Feedback: [::c_uchar; 16],
}
pub type PCRYPT_AES_256_KEY_STATE = *mut CRYPT_AES_256_KEY_STATE;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPTOAPI_BLOB {
    pub cbData: ::DWORD,
    pub pbData: *mut ::BYTE,
}
pub type CRYPT_INTEGER_BLOB = CRYPTOAPI_BLOB;
pub type PCRYPT_INTEGER_BLOB = *mut CRYPTOAPI_BLOB;
pub type CRYPT_UINT_BLOB = CRYPTOAPI_BLOB;
pub type PCRYPT_UINT_BLOB = *mut CRYPTOAPI_BLOB;
pub type CRYPT_OBJID_BLOB = CRYPTOAPI_BLOB;
pub type PCRYPT_OBJID_BLOB = *mut CRYPTOAPI_BLOB;
pub type CERT_NAME_BLOB = CRYPTOAPI_BLOB;
pub type PCERT_NAME_BLOB = *mut CRYPTOAPI_BLOB;
pub type CERT_RDN_VALUE_BLOB = CRYPTOAPI_BLOB;
pub type PCERT_RDN_VALUE_BLOB = *mut CRYPTOAPI_BLOB;
pub type CERT_BLOB = CRYPTOAPI_BLOB;
pub type PCERT_BLOB = *mut CRYPTOAPI_BLOB;
pub type CRL_BLOB = CRYPTOAPI_BLOB;
pub type PCRL_BLOB = *mut CRYPTOAPI_BLOB;
pub type DATA_BLOB = CRYPTOAPI_BLOB;
pub type PDATA_BLOB = *mut CRYPTOAPI_BLOB;
pub type CRYPT_DATA_BLOB = CRYPTOAPI_BLOB;
pub type PCRYPT_DATA_BLOB = *mut CRYPTOAPI_BLOB;
pub type CRYPT_HASH_BLOB = CRYPTOAPI_BLOB;
pub type PCRYPT_HASH_BLOB = *mut CRYPTOAPI_BLOB;
pub type CRYPT_DIGEST_BLOB = CRYPTOAPI_BLOB;
pub type PCRYPT_DIGEST_BLOB = *mut CRYPTOAPI_BLOB;
pub type CRYPT_DER_BLOB = CRYPTOAPI_BLOB;
pub type PCRYPT_DER_BLOB = *mut CRYPTOAPI_BLOB;
pub type CRYPT_ATTR_BLOB = CRYPTOAPI_BLOB;
pub type PCRYPT_ATTR_BLOB = *mut CRYPTOAPI_BLOB;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CMS_DH_KEY_INFO {
    pub dwVersion: ::DWORD,
    pub Algid: ALG_ID,
    pub pszContentEncObjId: ::LPSTR,
    pub PubInfo: CRYPT_DATA_BLOB,
    pub pReserved: *mut ::c_void,
}
pub type PCMS_DH_KEY_INFO = *mut CMS_DH_KEY_INFO;
pub type HCRYPTPROV_OR_NCRYPT_KEY_HANDLE = ::ULONG_PTR;
pub type HCRYPTPROV_LEGACY = ::ULONG_PTR;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_BIT_BLOB {
    pub cbData: ::DWORD,
    pub pbData: *mut ::BYTE,
    pub cUnusedBits: ::DWORD,
}
pub type PCRYPT_BIT_BLOB = *mut CRYPT_BIT_BLOB;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_ALGORITHM_IDENTIFIER {
    pub pszObjId: ::LPSTR,
    pub Parameters: CRYPT_OBJID_BLOB,
}
pub type PCRYPT_ALGORITHM_IDENTIFIER = *mut CRYPT_ALGORITHM_IDENTIFIER;
pub const szOID_RSA: &'static str = "1.2.840.113549";
pub const szOID_PKCS: &'static str = "1.2.840.113549.1";
pub const szOID_RSA_HASH: &'static str = "1.2.840.113549.2";
pub const szOID_RSA_ENCRYPT: &'static str = "1.2.840.113549.3";
pub const szOID_PKCS_1: &'static str = "1.2.840.113549.1.1";
pub const szOID_PKCS_2: &'static str = "1.2.840.113549.1.2";
pub const szOID_PKCS_3: &'static str = "1.2.840.113549.1.3";
pub const szOID_PKCS_4: &'static str = "1.2.840.113549.1.4";
pub const szOID_PKCS_5: &'static str = "1.2.840.113549.1.5";
pub const szOID_PKCS_6: &'static str = "1.2.840.113549.1.6";
pub const szOID_PKCS_7: &'static str = "1.2.840.113549.1.7";
pub const szOID_PKCS_8: &'static str = "1.2.840.113549.1.8";
pub const szOID_PKCS_9: &'static str = "1.2.840.113549.1.9";
pub const szOID_PKCS_10: &'static str = "1.2.840.113549.1.10";
pub const szOID_PKCS_12: &'static str = "1.2.840.113549.1.12";
pub const szOID_RSA_RSA: &'static str = "1.2.840.113549.1.1.1";
pub const szOID_RSA_MD2RSA: &'static str = "1.2.840.113549.1.1.2";
pub const szOID_RSA_MD4RSA: &'static str = "1.2.840.113549.1.1.3";
pub const szOID_RSA_MD5RSA: &'static str = "1.2.840.113549.1.1.4";
pub const szOID_RSA_SHA1RSA: &'static str = "1.2.840.113549.1.1.5";
pub const szOID_RSA_SETOAEP_RSA: &'static str = "1.2.840.113549.1.1.6";
pub const szOID_RSAES_OAEP: &'static str = "1.2.840.113549.1.1.7";
pub const szOID_RSA_MGF1: &'static str = "1.2.840.113549.1.1.8";
pub const szOID_RSA_PSPECIFIED: &'static str = "1.2.840.113549.1.1.9";
pub const szOID_RSA_SSA_PSS: &'static str = "1.2.840.113549.1.1.10";
pub const szOID_RSA_SHA256RSA: &'static str = "1.2.840.113549.1.1.11";
pub const szOID_RSA_SHA384RSA: &'static str = "1.2.840.113549.1.1.12";
pub const szOID_RSA_SHA512RSA: &'static str = "1.2.840.113549.1.1.13";
pub const szOID_RSA_DH: &'static str = "1.2.840.113549.1.3.1";
pub const szOID_RSA_data: &'static str = "1.2.840.113549.1.7.1";
pub const szOID_RSA_signedData: &'static str = "1.2.840.113549.1.7.2";
pub const szOID_RSA_envelopedData: &'static str = "1.2.840.113549.1.7.3";
pub const szOID_RSA_signEnvData: &'static str = "1.2.840.113549.1.7.4";
pub const szOID_RSA_digestedData: &'static str = "1.2.840.113549.1.7.5";
pub const szOID_RSA_hashedData: &'static str = "1.2.840.113549.1.7.5";
pub const szOID_RSA_encryptedData: &'static str = "1.2.840.113549.1.7.6";
pub const szOID_RSA_emailAddr: &'static str = "1.2.840.113549.1.9.1";
pub const szOID_RSA_unstructName: &'static str = "1.2.840.113549.1.9.2";
pub const szOID_RSA_contentType: &'static str = "1.2.840.113549.1.9.3";
pub const szOID_RSA_messageDigest: &'static str = "1.2.840.113549.1.9.4";
pub const szOID_RSA_signingTime: &'static str = "1.2.840.113549.1.9.5";
pub const szOID_RSA_counterSign: &'static str = "1.2.840.113549.1.9.6";
pub const szOID_RSA_challengePwd: &'static str = "1.2.840.113549.1.9.7";
pub const szOID_RSA_unstructAddr: &'static str = "1.2.840.113549.1.9.8";
pub const szOID_RSA_extCertAttrs: &'static str = "1.2.840.113549.1.9.9";
pub const szOID_RSA_certExtensions: &'static str = "1.2.840.113549.1.9.14";
pub const szOID_RSA_SMIMECapabilities: &'static str = "1.2.840.113549.1.9.15";
pub const szOID_RSA_preferSignedData: &'static str = "1.2.840.113549.1.9.15.1";
pub const szOID_TIMESTAMP_TOKEN: &'static str = "1.2.840.113549.1.9.16.1.4";
pub const szOID_RFC3161_counterSign: &'static str = "1.3.6.1.4.1.311.3.3.1";
pub const szOID_RSA_SMIMEalg: &'static str = "1.2.840.113549.1.9.16.3";
pub const szOID_RSA_SMIMEalgESDH: &'static str = "1.2.840.113549.1.9.16.3.5";
pub const szOID_RSA_SMIMEalgCMS3DESwrap: &'static str = "1.2.840.113549.1.9.16.3.6";
pub const szOID_RSA_SMIMEalgCMSRC2wrap: &'static str = "1.2.840.113549.1.9.16.3.7";
pub const szOID_RSA_MD2: &'static str = "1.2.840.113549.2.2";
pub const szOID_RSA_MD4: &'static str = "1.2.840.113549.2.4";
pub const szOID_RSA_MD5: &'static str = "1.2.840.113549.2.5";
pub const szOID_RSA_RC2CBC: &'static str = "1.2.840.113549.3.2";
pub const szOID_RSA_RC4: &'static str = "1.2.840.113549.3.4";
pub const szOID_RSA_DES_EDE3_CBC: &'static str = "1.2.840.113549.3.7";
pub const szOID_RSA_RC5_CBCPad: &'static str = "1.2.840.113549.3.9";
pub const szOID_ANSI_X942: &'static str = "1.2.840.10046";
pub const szOID_ANSI_X942_DH: &'static str = "1.2.840.10046.2.1";
pub const szOID_X957: &'static str = "1.2.840.10040";
pub const szOID_X957_DSA: &'static str = "1.2.840.10040.4.1";
pub const szOID_X957_SHA1DSA: &'static str = "1.2.840.10040.4.3";
pub const szOID_ECC_PUBLIC_KEY: &'static str = "1.2.840.10045.2.1";
pub const szOID_ECC_CURVE_P256: &'static str = "1.2.840.10045.3.1.7";
pub const szOID_ECC_CURVE_P384: &'static str = "1.3.132.0.34";
pub const szOID_ECC_CURVE_P521: &'static str = "1.3.132.0.35";
pub const szOID_ECDSA_SHA1: &'static str = "1.2.840.10045.4.1";
pub const szOID_ECDSA_SPECIFIED: &'static str = "1.2.840.10045.4.3";
pub const szOID_ECDSA_SHA256: &'static str = "1.2.840.10045.4.3.2";
pub const szOID_ECDSA_SHA384: &'static str = "1.2.840.10045.4.3.3";
pub const szOID_ECDSA_SHA512: &'static str = "1.2.840.10045.4.3.4";
pub const szOID_NIST_AES128_CBC: &'static str = "2.16.840.1.101.3.4.1.2";
pub const szOID_NIST_AES192_CBC: &'static str = "2.16.840.1.101.3.4.1.22";
pub const szOID_NIST_AES256_CBC: &'static str = "2.16.840.1.101.3.4.1.42";
pub const szOID_NIST_AES128_WRAP: &'static str = "2.16.840.1.101.3.4.1.5";
pub const szOID_NIST_AES192_WRAP: &'static str = "2.16.840.1.101.3.4.1.25";
pub const szOID_NIST_AES256_WRAP: &'static str = "2.16.840.1.101.3.4.1.45";
pub const szOID_DH_SINGLE_PASS_STDDH_SHA1_KDF: &'static str = "1.3.133.16.840.63.0.2";
pub const szOID_DH_SINGLE_PASS_STDDH_SHA256_KDF: &'static str = "1.3.132.1.11.1";
pub const szOID_DH_SINGLE_PASS_STDDH_SHA384_KDF: &'static str = "1.3.132.1.11.2";
pub const szOID_DS: &'static str = "2.5";
pub const szOID_DSALG: &'static str = "2.5.8";
pub const szOID_DSALG_CRPT: &'static str = "2.5.8.1";
pub const szOID_DSALG_HASH: &'static str = "2.5.8.2";
pub const szOID_DSALG_SIGN: &'static str = "2.5.8.3";
pub const szOID_DSALG_RSA: &'static str = "2.5.8.1.1";
pub const szOID_OIW: &'static str = "1.3.14";
pub const szOID_OIWSEC: &'static str = "1.3.14.3.2";
pub const szOID_OIWSEC_md4RSA: &'static str = "1.3.14.3.2.2";
pub const szOID_OIWSEC_md5RSA: &'static str = "1.3.14.3.2.3";
pub const szOID_OIWSEC_md4RSA2: &'static str = "1.3.14.3.2.4";
pub const szOID_OIWSEC_desECB: &'static str = "1.3.14.3.2.6";
pub const szOID_OIWSEC_desCBC: &'static str = "1.3.14.3.2.7";
pub const szOID_OIWSEC_desOFB: &'static str = "1.3.14.3.2.8";
pub const szOID_OIWSEC_desCFB: &'static str = "1.3.14.3.2.9";
pub const szOID_OIWSEC_desMAC: &'static str = "1.3.14.3.2.10";
pub const szOID_OIWSEC_rsaSign: &'static str = "1.3.14.3.2.11";
pub const szOID_OIWSEC_dsa: &'static str = "1.3.14.3.2.12";
pub const szOID_OIWSEC_shaDSA: &'static str = "1.3.14.3.2.13";
pub const szOID_OIWSEC_mdc2RSA: &'static str = "1.3.14.3.2.14";
pub const szOID_OIWSEC_shaRSA: &'static str = "1.3.14.3.2.15";
pub const szOID_OIWSEC_dhCommMod: &'static str = "1.3.14.3.2.16";
pub const szOID_OIWSEC_desEDE: &'static str = "1.3.14.3.2.17";
pub const szOID_OIWSEC_sha: &'static str = "1.3.14.3.2.18";
pub const szOID_OIWSEC_mdc2: &'static str = "1.3.14.3.2.19";
pub const szOID_OIWSEC_dsaComm: &'static str = "1.3.14.3.2.20";
pub const szOID_OIWSEC_dsaCommSHA: &'static str = "1.3.14.3.2.21";
pub const szOID_OIWSEC_rsaXchg: &'static str = "1.3.14.3.2.22";
pub const szOID_OIWSEC_keyHashSeal: &'static str = "1.3.14.3.2.23";
pub const szOID_OIWSEC_md2RSASign: &'static str = "1.3.14.3.2.24";
pub const szOID_OIWSEC_md5RSASign: &'static str = "1.3.14.3.2.25";
pub const szOID_OIWSEC_sha1: &'static str = "1.3.14.3.2.26";
pub const szOID_OIWSEC_dsaSHA1: &'static str = "1.3.14.3.2.27";
pub const szOID_OIWSEC_dsaCommSHA1: &'static str = "1.3.14.3.2.28";
pub const szOID_OIWSEC_sha1RSASign: &'static str = "1.3.14.3.2.29";
pub const szOID_OIWDIR: &'static str = "1.3.14.7.2";
pub const szOID_OIWDIR_CRPT: &'static str = "1.3.14.7.2.1";
pub const szOID_OIWDIR_HASH: &'static str = "1.3.14.7.2.2";
pub const szOID_OIWDIR_SIGN: &'static str = "1.3.14.7.2.3";
pub const szOID_OIWDIR_md2: &'static str = "1.3.14.7.2.2.1";
pub const szOID_OIWDIR_md2RSA: &'static str = "1.3.14.7.2.3.1";
pub const szOID_INFOSEC: &'static str = "2.16.840.1.101.2.1";
pub const szOID_INFOSEC_sdnsSignature: &'static str = "2.16.840.1.101.2.1.1.1";
pub const szOID_INFOSEC_mosaicSignature: &'static str = "2.16.840.1.101.2.1.1.2";
pub const szOID_INFOSEC_sdnsConfidentiality: &'static str = "2.16.840.1.101.2.1.1.3";
pub const szOID_INFOSEC_mosaicConfidentiality: &'static str = "2.16.840.1.101.2.1.1.4";
pub const szOID_INFOSEC_sdnsIntegrity: &'static str = "2.16.840.1.101.2.1.1.5";
pub const szOID_INFOSEC_mosaicIntegrity: &'static str = "2.16.840.1.101.2.1.1.6";
pub const szOID_INFOSEC_sdnsTokenProtection: &'static str = "2.16.840.1.101.2.1.1.7";
pub const szOID_INFOSEC_mosaicTokenProtection: &'static str = "2.16.840.1.101.2.1.1.8";
pub const szOID_INFOSEC_sdnsKeyManagement: &'static str = "2.16.840.1.101.2.1.1.9";
pub const szOID_INFOSEC_mosaicKeyManagement: &'static str = "2.16.840.1.101.2.1.1.10";
pub const szOID_INFOSEC_sdnsKMandSig: &'static str = "2.16.840.1.101.2.1.1.11";
pub const szOID_INFOSEC_mosaicKMandSig: &'static str = "2.16.840.1.101.2.1.1.12";
pub const szOID_INFOSEC_SuiteASignature: &'static str = "2.16.840.1.101.2.1.1.13";
pub const szOID_INFOSEC_SuiteAConfidentiality: &'static str = "2.16.840.1.101.2.1.1.14";
pub const szOID_INFOSEC_SuiteAIntegrity: &'static str = "2.16.840.1.101.2.1.1.15";
pub const szOID_INFOSEC_SuiteATokenProtection: &'static str = "2.16.840.1.101.2.1.1.16";
pub const szOID_INFOSEC_SuiteAKeyManagement: &'static str = "2.16.840.1.101.2.1.1.17";
pub const szOID_INFOSEC_SuiteAKMandSig: &'static str = "2.16.840.1.101.2.1.1.18";
pub const szOID_INFOSEC_mosaicUpdatedSig: &'static str = "2.16.840.1.101.2.1.1.19";
pub const szOID_INFOSEC_mosaicKMandUpdSig: &'static str = "2.16.840.1.101.2.1.1.20";
pub const szOID_INFOSEC_mosaicUpdatedInteg: &'static str = "2.16.840.1.101.2.1.1.21";
pub const szOID_NIST_sha256: &'static str = "2.16.840.1.101.3.4.2.1";
pub const szOID_NIST_sha384: &'static str = "2.16.840.1.101.3.4.2.2";
pub const szOID_NIST_sha512: &'static str = "2.16.840.1.101.3.4.2.3";
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_OBJID_TABLE {
    pub dwAlgId: ::DWORD,
    pub pszObjId: ::LPCSTR,
}
pub type PCRYPT_OBJID_TABLE = *mut CRYPT_OBJID_TABLE;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_HASH_INFO {
    pub HashAlgorithm: CRYPT_ALGORITHM_IDENTIFIER,
    pub Hash: CRYPT_HASH_BLOB,
}
pub type PCRYPT_HASH_INFO = *mut CRYPT_HASH_INFO;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_EXTENSION {
    pub pszObjId: ::LPSTR,
    pub fCritical: ::BOOL,
    pub Value: CRYPT_OBJID_BLOB,
}
pub type PCERT_EXTENSION = *mut CERT_EXTENSION;
pub type PCCERT_EXTENSION = *const CERT_EXTENSION;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_ATTRIBUTE_TYPE_VALUE {
    pub pszObjId: ::LPSTR,
    pub Value: CRYPT_OBJID_BLOB,
}
pub type PCRYPT_ATTRIBUTE_TYPE_VALUE = *mut CRYPT_ATTRIBUTE_TYPE_VALUE;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_ATTRIBUTE {
    pub pszObjId: ::LPSTR,
    pub cValue: ::DWORD,
    pub rgValue: PCRYPT_ATTR_BLOB,
}
pub type PCRYPT_ATTRIBUTE = *mut CRYPT_ATTRIBUTE;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_ATTRIBUTES {
    pub cAttr: ::DWORD,
    pub rgAttr: PCRYPT_ATTRIBUTE,
}
pub type PCRYPT_ATTRIBUTES = *mut CRYPT_ATTRIBUTES;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_RDN_ATTR {
    pub pszObjId: ::LPSTR,
    pub dwValueType: ::DWORD,
    pub Value: CERT_RDN_VALUE_BLOB,
}
pub type PCERT_RDN_ATTR = *mut CERT_RDN_ATTR;
pub const szOID_COMMON_NAME: &'static str = "2.5.4.3";
pub const szOID_SUR_NAME: &'static str = "2.5.4.4";
pub const szOID_DEVICE_SERIAL_NUMBER: &'static str = "2.5.4.5";
pub const szOID_COUNTRY_NAME: &'static str = "2.5.4.6";
pub const szOID_LOCALITY_NAME: &'static str = "2.5.4.7";
pub const szOID_STATE_OR_PROVINCE_NAME: &'static str = "2.5.4.8";
pub const szOID_STREET_ADDRESS: &'static str = "2.5.4.9";
pub const szOID_ORGANIZATION_NAME: &'static str = "2.5.4.10";
pub const szOID_ORGANIZATIONAL_UNIT_NAME: &'static str = "2.5.4.11";
pub const szOID_TITLE: &'static str = "2.5.4.12";
pub const szOID_DESCRIPTION: &'static str = "2.5.4.13";
pub const szOID_SEARCH_GUIDE: &'static str = "2.5.4.14";
pub const szOID_BUSINESS_CATEGORY: &'static str = "2.5.4.15";
pub const szOID_POSTAL_ADDRESS: &'static str = "2.5.4.16";
pub const szOID_POSTAL_CODE: &'static str = "2.5.4.17";
pub const szOID_POST_OFFICE_BOX: &'static str = "2.5.4.18";
pub const szOID_PHYSICAL_DELIVERY_OFFICE_NAME: &'static str = "2.5.4.19";
pub const szOID_TELEPHONE_NUMBER: &'static str = "2.5.4.20";
pub const szOID_TELEX_NUMBER: &'static str = "2.5.4.21";
pub const szOID_TELETEXT_TERMINAL_IDENTIFIER: &'static str = "2.5.4.22";
pub const szOID_FACSIMILE_TELEPHONE_NUMBER: &'static str = "2.5.4.23";
pub const szOID_X21_ADDRESS: &'static str = "2.5.4.24";
pub const szOID_INTERNATIONAL_ISDN_NUMBER: &'static str = "2.5.4.25";
pub const szOID_REGISTERED_ADDRESS: &'static str = "2.5.4.26";
pub const szOID_DESTINATION_INDICATOR: &'static str = "2.5.4.27";
pub const szOID_PREFERRED_DELIVERY_METHOD: &'static str = "2.5.4.28";
pub const szOID_PRESENTATION_ADDRESS: &'static str = "2.5.4.29";
pub const szOID_SUPPORTED_APPLICATION_CONTEXT: &'static str = "2.5.4.30";
pub const szOID_MEMBER: &'static str = "2.5.4.31";
pub const szOID_OWNER: &'static str = "2.5.4.32";
pub const szOID_ROLE_OCCUPANT: &'static str = "2.5.4.33";
pub const szOID_SEE_ALSO: &'static str = "2.5.4.34";
pub const szOID_USER_PASSWORD: &'static str = "2.5.4.35";
pub const szOID_USER_CERTIFICATE: &'static str = "2.5.4.36";
pub const szOID_CA_CERTIFICATE: &'static str = "2.5.4.37";
pub const szOID_AUTHORITY_REVOCATION_LIST: &'static str = "2.5.4.38";
pub const szOID_CERTIFICATE_REVOCATION_LIST: &'static str = "2.5.4.39";
pub const szOID_CROSS_CERTIFICATE_PAIR: &'static str = "2.5.4.40";
pub const szOID_GIVEN_NAME: &'static str = "2.5.4.42";
pub const szOID_INITIALS: &'static str = "2.5.4.43";
pub const szOID_DN_QUALIFIER: &'static str = "2.5.4.46";
pub const szOID_DOMAIN_COMPONENT: &'static str = "0.9.2342.19200300.100.1.25";
pub const szOID_PKCS_12_FRIENDLY_NAME_ATTR: &'static str = "1.2.840.113549.1.9.20";
pub const szOID_PKCS_12_LOCAL_KEY_ID: &'static str = "1.2.840.113549.1.9.21";
pub const szOID_PKCS_12_KEY_PROVIDER_NAME_ATTR: &'static str = "1.3.6.1.4.1.311.17.1";
pub const szOID_LOCAL_MACHINE_KEYSET: &'static str = "1.3.6.1.4.1.311.17.2";
pub const szOID_PKCS_12_EXTENDED_ATTRIBUTES: &'static str = "1.3.6.1.4.1.311.17.3";
pub const szOID_PKCS_12_PROTECTED_PASSWORD_SECRET_BAG_TYPE_ID: &'static str =
    "1.3.6.1.4.1.311.17.4";
pub const szOID_KEYID_RDN: &'static str = "1.3.6.1.4.1.311.10.7.1";
pub const szOID_EV_RDN_LOCALE: &'static str = "1.3.6.1.4.1.311.60.2.1.1";
pub const szOID_EV_RDN_STATE_OR_PROVINCE: &'static str = "1.3.6.1.4.1.311.60.2.1.2";
pub const szOID_EV_RDN_COUNTRY: &'static str = "1.3.6.1.4.1.311.60.2.1.3";
pub const CERT_RDN_ANY_TYPE: ::DWORD = 0;
pub const CERT_RDN_ENCODED_BLOB: ::DWORD = 1;
pub const CERT_RDN_OCTET_STRING: ::DWORD = 2;
pub const CERT_RDN_NUMERIC_STRING: ::DWORD = 3;
pub const CERT_RDN_PRINTABLE_STRING: ::DWORD = 4;
pub const CERT_RDN_TELETEX_STRING: ::DWORD = 5;
pub const CERT_RDN_T61_STRING: ::DWORD = 5;
pub const CERT_RDN_VIDEOTEX_STRING: ::DWORD = 6;
pub const CERT_RDN_IA5_STRING: ::DWORD = 7;
pub const CERT_RDN_GRAPHIC_STRING: ::DWORD = 8;
pub const CERT_RDN_VISIBLE_STRING: ::DWORD = 9;
pub const CERT_RDN_ISO646_STRING: ::DWORD = 9;
pub const CERT_RDN_GENERAL_STRING: ::DWORD = 10;
pub const CERT_RDN_UNIVERSAL_STRING: ::DWORD = 11;
pub const CERT_RDN_INT4_STRING: ::DWORD = 11;
pub const CERT_RDN_BMP_STRING: ::DWORD = 12;
pub const CERT_RDN_UNICODE_STRING: ::DWORD = 12;
pub const CERT_RDN_UTF8_STRING: ::DWORD = 13;
pub const CERT_RDN_TYPE_MASK: ::DWORD = 0x000000FF;
pub const CERT_RDN_FLAGS_MASK: ::DWORD = 0xFF000000;
pub const CERT_RDN_ENABLE_T61_UNICODE_FLAG: ::DWORD = 0x80000000;
pub const CERT_RDN_ENABLE_UTF8_UNICODE_FLAG: ::DWORD = 0x20000000;
pub const CERT_RDN_FORCE_UTF8_UNICODE_FLAG: ::DWORD = 0x10000000;
pub const CERT_RDN_DISABLE_CHECK_TYPE_FLAG: ::DWORD = 0x40000000;
pub const CERT_RDN_DISABLE_IE4_UTF8_FLAG: ::DWORD = 0x01000000;
pub const CERT_RDN_ENABLE_PUNYCODE_FLAG: ::DWORD = 0x02000000;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_RDN {
    pub cRDNAttr: ::DWORD,
    pub rgRDNAttr: PCERT_RDN_ATTR,
}
pub type PCERT_RDN = *mut CERT_RDN;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_NAME_INFO {
    pub cRDN: ::DWORD,
    pub rgRDN: PCERT_RDN,
}
pub type PCERT_NAME_INFO = *mut CERT_NAME_INFO;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_NAME_VALUE {
    pub dwValueType: ::DWORD,
    pub Value: CERT_RDN_VALUE_BLOB,
}
pub type PCERT_NAME_VALUE = *mut CERT_NAME_VALUE;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_PUBLIC_KEY_INFO {
    pub Algorithm: CRYPT_ALGORITHM_IDENTIFIER,
    pub PublicKey: CRYPT_BIT_BLOB,
}
pub type PCERT_PUBLIC_KEY_INFO = *mut CERT_PUBLIC_KEY_INFO;
pub const CERT_RSA_PUBLIC_KEY_OBJID: &'static str = szOID_RSA_RSA;
pub const CERT_DEFAULT_OID_PUBLIC_KEY_SIGN: &'static str = szOID_RSA_RSA;
pub const CERT_DEFAULT_OID_PUBLIC_KEY_XCHG: &'static str = szOID_RSA_RSA;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_ECC_PRIVATE_KEY_INFO {
    pub dwVersion: ::DWORD,
    pub PrivateKey: CRYPT_DER_BLOB,
    pub szCurveOid: ::LPSTR,
    pub PublicKey: CRYPT_BIT_BLOB,
}
pub type PCRYPT_ECC_PRIVATE_KEY_INFO = *mut CRYPT_ECC_PRIVATE_KEY_INFO;
pub const CRYPT_ECC_PRIVATE_KEY_INFO_v1: ::DWORD = 1;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_PRIVATE_KEY_INFO {
    pub Version: ::DWORD,
    pub Algorithm: CRYPT_ALGORITHM_IDENTIFIER,
    pub PrivateKey: CRYPT_DER_BLOB,
    pub pAttributes: PCRYPT_ATTRIBUTES,
}
pub type PCRYPT_PRIVATE_KEY_INFO = *mut CRYPT_PRIVATE_KEY_INFO;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_ENCRYPTED_PRIVATE_KEY_INFO {
    pub EncryptionAlgorithm: ::CRYPT_ALGORITHM_IDENTIFIER,
    pub EncryptedPrivateKey: ::CRYPT_DATA_BLOB,
}
pub type PCRYPT_ENCRYPTED_PRIVATE_KEY_INFO = *mut CRYPT_ENCRYPTED_PRIVATE_KEY_INFO;
pub type PCRYPT_DECRYPT_PRIVATE_KEY_FUNC = Option<unsafe extern "system" fn(
    Algorithm: CRYPT_ALGORITHM_IDENTIFIER, EncryptedPrivateKey: CRYPT_DATA_BLOB,
    pbClearTextKey: *mut ::BYTE, pcbClearTextKey: *mut ::DWORD, pVoidDecryptFunc: ::LPVOID,
) -> ::BOOL>;
pub type PCRYPT_ENCRYPT_PRIVATE_KEY_FUNC = Option<unsafe extern "system" fn(
    Algorithm: *mut CRYPT_ALGORITHM_IDENTIFIER, pClearTextPrivateKey: *mut CRYPT_DATA_BLOB,
    pbEncryptedKey: *mut ::BYTE, pcbEncryptedKey: *mut ::DWORD, pVoidEncryptFunc: ::LPVOID,
) -> ::BOOL>;
pub type PCRYPT_RESOLVE_HCRYPTPROV_FUNC = Option<unsafe extern "system" fn(
    pPrivateKeyInfo: *mut CRYPT_PRIVATE_KEY_INFO, phCryptProv: *mut HCRYPTPROV,
    pVoidResolveFunc: ::LPVOID,
) -> ::BOOL>;
#[repr(C)] #[derive(Copy)]
pub struct CRYPT_PKCS8_IMPORT_PARAMS {
    pub PrivateKey: CRYPT_DIGEST_BLOB,
    pub pResolvehCryptProvFunc: PCRYPT_RESOLVE_HCRYPTPROV_FUNC,
    pub pVoidResolveFunc: ::LPVOID,
    pub pDecryptPrivateKeyFunc: PCRYPT_DECRYPT_PRIVATE_KEY_FUNC,
    pub pVoidDecryptFunc: ::LPVOID,
}
impl Clone for CRYPT_PKCS8_IMPORT_PARAMS { fn clone(&self) -> CRYPT_PKCS8_IMPORT_PARAMS { *self } }
pub type PCRYPT_PKCS8_IMPORT_PARAMS = *mut CRYPT_PKCS8_IMPORT_PARAMS;
pub type CRYPT_PRIVATE_KEY_BLOB_AND_PARAMS = CRYPT_PKCS8_IMPORT_PARAMS;
pub type PPCRYPT_PRIVATE_KEY_BLOB_AND_PARAMS = *mut CRYPT_PKCS8_IMPORT_PARAMS;
#[repr(C)] #[derive(Copy)]
pub struct CRYPT_PKCS8_EXPORT_PARAMS {
    pub hCryptProv: HCRYPTPROV,
    pub dwKeySpec: ::DWORD,
    pub pszPrivateKeyObjId: ::LPSTR,
    pub pEncryptPrivateKeyFunc: PCRYPT_ENCRYPT_PRIVATE_KEY_FUNC,
    pub pVoidEncryptFunc: ::LPVOID,
}
impl Clone for CRYPT_PKCS8_EXPORT_PARAMS { fn clone(&self) -> CRYPT_PKCS8_EXPORT_PARAMS { *self } }
pub type PCRYPT_PKCS8_EXPORT_PARAMS = *mut CRYPT_PKCS8_EXPORT_PARAMS;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_INFO {
    pub dwVersion: ::DWORD,
    pub SerialNumber: CRYPT_INTEGER_BLOB,
    pub SignatureAlgorithm: CRYPT_ALGORITHM_IDENTIFIER,
    pub Issuer: CERT_NAME_BLOB,
    pub NotBefore: ::FILETIME,
    pub NotAfter: ::FILETIME,
    pub Subject: CERT_NAME_BLOB,
    pub SubjectPublicKeyInfo: CERT_PUBLIC_KEY_INFO,
    pub IssuerUniqueId: CRYPT_BIT_BLOB,
    pub SubjectUniqueId: CRYPT_BIT_BLOB,
    pub cExtension: ::DWORD,
    pub rgExtension: PCERT_EXTENSION,
}
pub type PCERT_INFO = *mut CERT_INFO;
pub const CERT_V1: ::DWORD = 0;
pub const CERT_V2: ::DWORD = 1;
pub const CERT_V3: ::DWORD = 2;
pub const CERT_INFO_VERSION_FLAG: ::DWORD = 1;
pub const CERT_INFO_SERIAL_NUMBER_FLAG: ::DWORD = 2;
pub const CERT_INFO_SIGNATURE_ALGORITHM_FLAG: ::DWORD = 3;
pub const CERT_INFO_ISSUER_FLAG: ::DWORD = 4;
pub const CERT_INFO_NOT_BEFORE_FLAG: ::DWORD = 5;
pub const CERT_INFO_NOT_AFTER_FLAG: ::DWORD = 6;
pub const CERT_INFO_SUBJECT_FLAG: ::DWORD = 7;
pub const CERT_INFO_SUBJECT_PUBLIC_KEY_INFO_FLAG: ::DWORD = 8;
pub const CERT_INFO_ISSUER_UNIQUE_ID_FLAG: ::DWORD = 9;
pub const CERT_INFO_SUBJECT_UNIQUE_ID_FLAG: ::DWORD = 10;
pub const CERT_INFO_EXTENSION_FLAG: ::DWORD = 11;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRL_ENTRY {
    pub SerialNumber: CRYPT_INTEGER_BLOB,
    pub RevocationDate: ::FILETIME,
    pub cExtension: ::DWORD,
    pub rgExtension: PCERT_EXTENSION,
}
pub type PCRL_ENTRY = *mut CRL_ENTRY;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRL_INFO {
    pub dwVersion: ::DWORD,
    pub SignatureAlgorithm: CRYPT_ALGORITHM_IDENTIFIER,
    pub Issuer: CERT_NAME_BLOB,
    pub ThisUpdate: ::FILETIME,
    pub NextUpdate: ::FILETIME,
    pub cCRLEntry: ::DWORD,
    pub rgCRLEntry: PCRL_ENTRY,
    pub cExtension: ::DWORD,
    pub rgExtension: PCERT_EXTENSION,
}
pub type PCRL_INFO = *mut CRL_INFO;
pub const CRL_V1: ::DWORD = 0;
pub const CRL_V2: ::DWORD = 1;
pub const CERT_BUNDLE_CERTIFICATE: ::DWORD = 0;
pub const CERT_BUNDLE_CRL: ::DWORD = 1;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_OR_CRL_BLOB {
    pub dwChoice: ::DWORD,
    pub cbEncoded: ::DWORD,
    pub pbEncoded: *mut ::BYTE,
}
pub type PCERT_OR_CRL_BLOB = *mut CERT_OR_CRL_BLOB;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_OR_CRL_BUNDLE {
    pub cItem: ::DWORD,
    pub rgItem: PCERT_OR_CRL_BLOB,
}
pub type PCERT_OR_CRL_BUNDLE = *mut CERT_OR_CRL_BUNDLE;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_REQUEST_INFO {
    pub dwVersion: ::DWORD,
    pub Subject: CERT_NAME_BLOB,
    pub SubjectPublicKeyInfo: CERT_PUBLIC_KEY_INFO,
    pub cAttribute: ::DWORD,
    pub rgAttribute: PCRYPT_ATTRIBUTE,
}
pub type PCERT_REQUEST_INFO = *mut CERT_REQUEST_INFO;
pub const CERT_REQUEST_V1: ::DWORD = 0;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_KEYGEN_REQUEST_INFO {
    pub dwVersion: ::DWORD,
    pub SubjectPublicKeyInfo: CERT_PUBLIC_KEY_INFO,
    pub pwszChallengeString: ::LPWSTR,
}
pub type PCERT_KEYGEN_REQUEST_INFO = *mut CERT_KEYGEN_REQUEST_INFO;
pub const CERT_KEYGEN_REQUEST_V1: ::DWORD = 0;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_SIGNED_CONTENT_INFO {
    pub ToBeSigned: CRYPT_DER_BLOB,
    pub SignatureAlgorithm: CRYPT_ALGORITHM_IDENTIFIER,
    pub Signature: CRYPT_BIT_BLOB,
}
pub type PCERT_SIGNED_CONTENT_INFO = *mut CERT_SIGNED_CONTENT_INFO;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CTL_USAGE {
    pub cUsageIdentifier: ::DWORD,
    pub rgpszUsageIdentifier: *mut ::LPSTR,
}
pub type PCTL_USAGE = *mut CTL_USAGE;
pub type CERT_ENHKEY_USAGE = CTL_USAGE;
pub type PCERT_ENHKEY_USAGE = *mut CERT_ENHKEY_USAGE;
pub type PCCTL_USAGE = *const CTL_USAGE;
pub type PCCERT_ENHKEY_USAGE = *const CERT_ENHKEY_USAGE;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CTL_ENTRY {
    pub SubjectIdentifier: CRYPT_DATA_BLOB,
    pub cAttribute: ::DWORD,
    pub rgAttribute: PCRYPT_ATTRIBUTE,
}
pub type PCTL_ENTRY = *mut CTL_ENTRY;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CTL_INFO {
    pub dwVersion: ::DWORD,
    pub SubjectUsage: CTL_USAGE,
    pub ListIdentifier: CRYPT_DATA_BLOB,
    pub SequenceNumber: CRYPT_INTEGER_BLOB,
    pub ThisUpdate: ::FILETIME,
    pub NextUpdate: ::FILETIME,
    pub SubjectAlgorithm: CRYPT_ALGORITHM_IDENTIFIER,
    pub cCTLEntry: ::DWORD,
    pub rgCTLEntry: PCTL_ENTRY,
    pub cExtension: ::DWORD,
    pub rgExtension: PCERT_EXTENSION,
}
pub type PCTL_INFO = *mut CTL_INFO;
pub const CTL_V1: ::DWORD = 0;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_TIME_STAMP_REQUEST_INFO {
    pub pszTimeStampAlgorithm: ::LPSTR,
    pub pszContentType: ::LPSTR,
    pub Content: CRYPT_OBJID_BLOB,
    pub cAttribute: ::DWORD,
    pub rgAttribute: PCRYPT_ATTRIBUTE,
}
pub type PCRYPT_TIME_STAMP_REQUEST_INFO = *mut CRYPT_TIME_STAMP_REQUEST_INFO;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_ENROLLMENT_NAME_VALUE_PAIR {
    pub pwszName: ::LPWSTR,
    pub pwszValue: ::LPWSTR,
}
pub type PCRYPT_ENROLLMENT_NAME_VALUE_PAIR = *mut CRYPT_ENROLLMENT_NAME_VALUE_PAIR;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CRYPT_CSP_PROVIDER {
    pub dwKeySpec: ::DWORD,
    pub pwszProviderName: ::LPWSTR,
    pub Signature: CRYPT_BIT_BLOB,
}
pub type PCRYPT_CSP_PROVIDER = *mut CRYPT_CSP_PROVIDER;
pub const CERT_ENCODING_TYPE_MASK: ::DWORD = 0x0000FFFF;
pub const CMSG_ENCODING_TYPE_MASK: ::DWORD = 0xFFFF0000;
pub const CRYPT_ASN_ENCODING: ::DWORD = 0x00000001;
pub const CRYPT_NDR_ENCODING: ::DWORD = 0x00000002;
pub const X509_ASN_ENCODING: ::DWORD = 0x00000001;
pub const X509_NDR_ENCODING: ::DWORD = 0x00000002;
pub const PKCS_7_ASN_ENCODING: ::DWORD = 0x00010000;
pub const PKCS_7_NDR_ENCODING: ::DWORD = 0x00020000;
pub const CRYPT_FORMAT_STR_MULTI_LINE: ::DWORD = 0x0001;
pub const CRYPT_FORMAT_STR_NO_HEX: ::DWORD = 0x0010;
pub const CRYPT_FORMAT_SIMPLE: ::DWORD = 0x0001;
pub const CRYPT_FORMAT_X509: ::DWORD = 0x0002;
pub const CRYPT_FORMAT_OID: ::DWORD = 0x0004;
pub const CRYPT_FORMAT_RDN_SEMICOLON: ::DWORD = 0x0100;
pub const CRYPT_FORMAT_RDN_CRLF: ::DWORD = 0x0200;
pub const CRYPT_FORMAT_RDN_UNQUOTE: ::DWORD = 0x0400;
pub const CRYPT_FORMAT_RDN_REVERSE: ::DWORD = 0x0800;
pub const CRYPT_FORMAT_COMMA: ::DWORD = 0x1000;
pub const CRYPT_FORMAT_SEMICOLON: ::DWORD = CRYPT_FORMAT_RDN_SEMICOLON;
pub const CRYPT_FORMAT_CRLF: ::DWORD = CRYPT_FORMAT_RDN_CRLF;
pub type PFN_CRYPT_ALLOC = Option<unsafe extern "system" fn(cbSize: ::size_t)>;
pub type PFN_CRYPT_FREE = Option<unsafe extern "system" fn(pv: ::LPVOID)>;
#[repr(C)] #[derive(Copy)]
pub struct CRYPT_ENCODE_PARA {
    pub cbSize: ::DWORD,
    pub pfnAlloc: PFN_CRYPT_ALLOC,
    pub pfnFree: PFN_CRYPT_FREE,
}
impl Clone for CRYPT_ENCODE_PARA { fn clone(&self) -> CRYPT_ENCODE_PARA { *self } }
pub type PCRYPT_ENCODE_PARA = *mut CRYPT_ENCODE_PARA;
pub const CRYPT_ENCODE_NO_SIGNATURE_BYTE_REVERSAL_FLAG: ::DWORD = 0x8;
pub const CRYPT_ENCODE_ALLOC_FLAG: ::DWORD = 0x8000;
pub const CRYPT_UNICODE_NAME_ENCODE_ENABLE_T61_UNICODE_FLAG: ::DWORD =
    CERT_RDN_ENABLE_T61_UNICODE_FLAG;
pub const CRYPT_UNICODE_NAME_ENCODE_ENABLE_UTF8_UNICODE_FLAG: ::DWORD =
    CERT_RDN_ENABLE_UTF8_UNICODE_FLAG;
pub const CRYPT_UNICODE_NAME_ENCODE_FORCE_UTF8_UNICODE_FLAG: ::DWORD =
    CERT_RDN_FORCE_UTF8_UNICODE_FLAG;
pub const CRYPT_UNICODE_NAME_ENCODE_DISABLE_CHECK_TYPE_FLAG: ::DWORD =
    CERT_RDN_DISABLE_CHECK_TYPE_FLAG;
pub const CRYPT_SORTED_CTL_ENCODE_HASHED_SUBJECT_IDENTIFIER_FLAG: ::DWORD = 0x10000;
pub const CRYPT_ENCODE_ENABLE_PUNYCODE_FLAG: ::DWORD = 0x20000;
pub const CRYPT_ENCODE_ENABLE_UTF8PERCENT_FLAG: ::DWORD = 0x40000;
pub const CRYPT_ENCODE_ENABLE_IA5CONVERSION_FLAG: ::DWORD = CRYPT_ENCODE_ENABLE_PUNYCODE_FLAG
    | CRYPT_ENCODE_ENABLE_UTF8PERCENT_FLAG;
#[repr(C)] #[derive(Copy)]
pub struct CRYPT_DECODE_PARA {
    pub cbSize: ::DWORD,
    pub pfnAlloc: PFN_CRYPT_ALLOC,
    pub pfnFree: PFN_CRYPT_FREE,
}
impl Clone for CRYPT_DECODE_PARA { fn clone(&self) -> CRYPT_DECODE_PARA { *self } }
pub type PCRYPT_DECODE_PARA = *mut CRYPT_DECODE_PARA;
pub const CRYPT_DECODE_NOCOPY_FLAG: ::DWORD = 0x1;
pub const CRYPT_DECODE_TO_BE_SIGNED_FLAG: ::DWORD = 0x2;
pub const CRYPT_DECODE_SHARE_OID_STRING_FLAG: ::DWORD = 0x4;
pub const CRYPT_DECODE_NO_SIGNATURE_BYTE_REVERSAL_FLAG: ::DWORD = 0x8;
pub const CRYPT_DECODE_ALLOC_FLAG: ::DWORD = 0x8000;
pub const CRYPT_UNICODE_NAME_DECODE_DISABLE_IE4_UTF8_FLAG: ::DWORD =
    CERT_RDN_DISABLE_IE4_UTF8_FLAG;
pub const CRYPT_DECODE_ENABLE_PUNYCODE_FLAG: ::DWORD = 0x02000000;
pub const CRYPT_DECODE_ENABLE_UTF8PERCENT_FLAG: ::DWORD = 0x04000000;
pub const CRYPT_DECODE_ENABLE_IA5CONVERSION_FLAG: ::DWORD = CRYPT_DECODE_ENABLE_PUNYCODE_FLAG
    | CRYPT_DECODE_ENABLE_UTF8PERCENT_FLAG;
pub const CRYPT_ENCODE_DECODE_NONE: ::LPCSTR = 0 as ::LPCSTR;
pub const X509_CERT: ::LPCSTR = 1 as ::LPCSTR;
pub const X509_CERT_TO_BE_SIGNED: ::LPCSTR = 2 as ::LPCSTR;
pub const X509_CERT_CRL_TO_BE_SIGNED: ::LPCSTR = 3 as ::LPCSTR;
pub const X509_CERT_REQUEST_TO_BE_SIGNED: ::LPCSTR = 4 as ::LPCSTR;
pub const X509_EXTENSIONS: ::LPCSTR = 5 as ::LPCSTR;
pub const X509_NAME_VALUE: ::LPCSTR = 6 as ::LPCSTR;
pub const X509_NAME: ::LPCSTR = 7 as ::LPCSTR;
pub const X509_PUBLIC_KEY_INFO: ::LPCSTR = 8 as ::LPCSTR;
pub const X509_AUTHORITY_KEY_ID: ::LPCSTR = 9 as ::LPCSTR;
pub const X509_KEY_ATTRIBUTES: ::LPCSTR = 10 as ::LPCSTR;
pub const X509_KEY_USAGE_RESTRICTION: ::LPCSTR = 11 as ::LPCSTR;
pub const X509_ALTERNATE_NAME: ::LPCSTR = 12 as ::LPCSTR;
pub const X509_BASIC_CONSTRAINTS: ::LPCSTR = 13 as ::LPCSTR;
pub const X509_KEY_USAGE: ::LPCSTR = 14 as ::LPCSTR;
pub const X509_BASIC_CONSTRAINTS2: ::LPCSTR = 15 as ::LPCSTR;
pub const X509_CERT_POLICIES: ::LPCSTR = 16 as ::LPCSTR;
pub const PKCS_UTC_TIME: ::LPCSTR = 17 as ::LPCSTR;
pub const PKCS_TIME_REQUEST: ::LPCSTR = 18 as ::LPCSTR;
pub const RSA_CSP_PUBLICKEYBLOB: ::LPCSTR = 19 as ::LPCSTR;
pub const X509_UNICODE_NAME: ::LPCSTR = 20 as ::LPCSTR;
pub const X509_KEYGEN_REQUEST_TO_BE_SIGNED: ::LPCSTR = 21 as ::LPCSTR;
pub const PKCS_ATTRIBUTE: ::LPCSTR = 22 as ::LPCSTR;
pub const PKCS_CONTENT_INFO_SEQUENCE_OF_ANY: ::LPCSTR = 23 as ::LPCSTR;
pub const X509_UNICODE_NAME_VALUE: ::LPCSTR = 24 as ::LPCSTR;
pub const X509_ANY_STRING: ::LPCSTR = X509_NAME_VALUE;
pub const X509_UNICODE_ANY_STRING: ::LPCSTR = X509_UNICODE_NAME_VALUE;
pub const X509_OCTET_STRING: ::LPCSTR = 25 as ::LPCSTR;
pub const X509_BITS: ::LPCSTR = 26 as ::LPCSTR;
pub const X509_INTEGER: ::LPCSTR = 27 as ::LPCSTR;
pub const X509_MULTI_BYTE_INTEGER: ::LPCSTR = 28 as ::LPCSTR;
pub const X509_ENUMERATED: ::LPCSTR = 29 as ::LPCSTR;
pub const X509_CHOICE_OF_TIME: ::LPCSTR = 30 as ::LPCSTR;
pub const X509_AUTHORITY_KEY_ID2: ::LPCSTR = 31 as ::LPCSTR;
pub const X509_AUTHORITY_INFO_ACCESS: ::LPCSTR = 32 as ::LPCSTR;
pub const X509_SUBJECT_INFO_ACCESS: ::LPCSTR = X509_AUTHORITY_INFO_ACCESS;
pub const X509_CRL_REASON_CODE: ::LPCSTR = X509_ENUMERATED;
pub const PKCS_CONTENT_INFO: ::LPCSTR = 33 as ::LPCSTR;
pub const X509_SEQUENCE_OF_ANY: ::LPCSTR = 34 as ::LPCSTR;
pub const X509_CRL_DIST_POINTS: ::LPCSTR = 35 as ::LPCSTR;
pub const X509_ENHANCED_KEY_USAGE: ::LPCSTR = 36 as ::LPCSTR;
pub const PKCS_CTL: ::LPCSTR = 37 as ::LPCSTR;
pub const X509_MULTI_BYTE_UINT: ::LPCSTR = 38 as ::LPCSTR;
pub const X509_DSS_PUBLICKEY: ::LPCSTR = X509_MULTI_BYTE_UINT;
pub const X509_DSS_PARAMETERS: ::LPCSTR = 39 as ::LPCSTR;
pub const X509_DSS_SIGNATURE: ::LPCSTR = 40 as ::LPCSTR;
pub const PKCS_RC2_CBC_PARAMETERS: ::LPCSTR = 41 as ::LPCSTR;
pub const PKCS_SMIME_CAPABILITIES: ::LPCSTR = 42 as ::LPCSTR;
pub const X509_QC_STATEMENTS_EXT: ::LPCSTR = 42 as ::LPCSTR;
pub const PKCS_RSA_PRIVATE_KEY: ::LPCSTR = 43 as ::LPCSTR;
pub const PKCS_PRIVATE_KEY_INFO: ::LPCSTR = 44 as ::LPCSTR;
pub const PKCS_ENCRYPTED_PRIVATE_KEY_INFO: ::LPCSTR = 45 as ::LPCSTR;
pub const X509_PKIX_POLICY_QUALIFIER_USERNOTICE: ::LPCSTR = 46 as ::LPCSTR;
pub const X509_DH_PUBLICKEY: ::LPCSTR = X509_MULTI_BYTE_UINT;
pub const X509_DH_PARAMETERS: ::LPCSTR = 47 as ::LPCSTR;
pub const PKCS_ATTRIBUTES: ::LPCSTR = 48 as ::LPCSTR;
pub const PKCS_SORTED_CTL: ::LPCSTR = 49 as ::LPCSTR;
pub const X509_ECC_SIGNATURE: ::LPCSTR = 47 as ::LPCSTR;
pub const X942_DH_PARAMETERS: ::LPCSTR = 50 as ::LPCSTR;
pub const X509_BITS_WITHOUT_TRAILING_ZEROES: ::LPCSTR = 51 as ::LPCSTR;
pub const X942_OTHER_INFO: ::LPCSTR = 52 as ::LPCSTR;
pub const X509_CERT_PAIR: ::LPCSTR = 53 as ::LPCSTR;
pub const X509_ISSUING_DIST_POINT: ::LPCSTR = 54 as ::LPCSTR;
pub const X509_NAME_CONSTRAINTS: ::LPCSTR = 55 as ::LPCSTR;
pub const X509_POLICY_MAPPINGS: ::LPCSTR = 56 as ::LPCSTR;
pub const X509_POLICY_CONSTRAINTS: ::LPCSTR = 57 as ::LPCSTR;
pub const X509_CROSS_CERT_DIST_POINTS: ::LPCSTR = 58 as ::LPCSTR;
pub const CMC_DATA: ::LPCSTR = 59 as ::LPCSTR;
pub const CMC_RESPONSE: ::LPCSTR = 60 as ::LPCSTR;
pub const CMC_STATUS: ::LPCSTR = 61 as ::LPCSTR;
pub const CMC_ADD_EXTENSIONS: ::LPCSTR = 62 as ::LPCSTR;
pub const CMC_ADD_ATTRIBUTES: ::LPCSTR = 63 as ::LPCSTR;
pub const X509_CERTIFICATE_TEMPLATE: ::LPCSTR = 64 as ::LPCSTR;
pub const OCSP_SIGNED_REQUEST: ::LPCSTR = 65 as ::LPCSTR;
pub const OCSP_REQUEST: ::LPCSTR = 66 as ::LPCSTR;
pub const OCSP_RESPONSE: ::LPCSTR = 67 as ::LPCSTR;
pub const OCSP_BASIC_SIGNED_RESPONSE: ::LPCSTR = 68 as ::LPCSTR;
pub const OCSP_BASIC_RESPONSE: ::LPCSTR = 69 as ::LPCSTR;
pub const X509_LOGOTYPE_EXT: ::LPCSTR = 70 as ::LPCSTR;
pub const X509_BIOMETRIC_EXT: ::LPCSTR = 71 as ::LPCSTR;
pub const CNG_RSA_PUBLIC_KEY_BLOB: ::LPCSTR = 72 as ::LPCSTR;
pub const X509_OBJECT_IDENTIFIER: ::LPCSTR = 73 as ::LPCSTR;
pub const X509_ALGORITHM_IDENTIFIER: ::LPCSTR = 74 as ::LPCSTR;
pub const PKCS_RSA_SSA_PSS_PARAMETERS: ::LPCSTR = 75 as ::LPCSTR;
pub const PKCS_RSAES_OAEP_PARAMETERS: ::LPCSTR = 76 as ::LPCSTR;
pub const ECC_CMS_SHARED_INFO: ::LPCSTR = 77 as ::LPCSTR;
pub const TIMESTAMP_REQUEST: ::LPCSTR = 78 as ::LPCSTR;
pub const TIMESTAMP_RESPONSE: ::LPCSTR = 79 as ::LPCSTR;
pub const TIMESTAMP_INFO: ::LPCSTR = 80 as ::LPCSTR;
pub const X509_CERT_BUNDLE: ::LPCSTR = 81 as ::LPCSTR;
pub const X509_ECC_PRIVATE_KEY: ::LPCSTR = 82 as ::LPCSTR;
pub const CNG_RSA_PRIVATE_KEY_BLOB: ::LPCSTR = 83 as ::LPCSTR;
pub const X509_SUBJECT_DIR_ATTRS: ::LPCSTR = 84 as ::LPCSTR;
pub const PKCS7_SIGNER_INFO: ::LPCSTR = 500 as ::LPCSTR;
pub const CMS_SIGNER_INFO: ::LPCSTR = 501 as ::LPCSTR;
pub const szOID_AUTHORITY_KEY_IDENTIFIER: &'static str = "2.5.29.1";
pub const szOID_KEY_ATTRIBUTES: &'static str = "2.5.29.2";
pub const szOID_CERT_POLICIES_95: &'static str = "2.5.29.3";
pub const szOID_KEY_USAGE_RESTRICTION: &'static str = "2.5.29.4";
pub const szOID_SUBJECT_ALT_NAME: &'static str = "2.5.29.7";
pub const szOID_ISSUER_ALT_NAME: &'static str = "2.5.29.8";
pub const szOID_BASIC_CONSTRAINTS: &'static str = "2.5.29.10";
pub const szOID_KEY_USAGE: &'static str = "2.5.29.15";
pub const szOID_PRIVATEKEY_USAGE_PERIOD: &'static str = "2.5.29.16";
pub const szOID_BASIC_CONSTRAINTS2: &'static str = "2.5.29.19";
pub const szOID_CERT_POLICIES: &'static str = "2.5.29.32";
pub const szOID_ANY_CERT_POLICY: &'static str = "2.5.29.32.0";
pub const szOID_INHIBIT_ANY_POLICY: &'static str = "2.5.29.54";
pub const szOID_AUTHORITY_KEY_IDENTIFIER2: &'static str = "2.5.29.35";
pub const szOID_SUBJECT_KEY_IDENTIFIER: &'static str = "2.5.29.14";
pub const szOID_SUBJECT_ALT_NAME2: &'static str = "2.5.29.17";
pub const szOID_ISSUER_ALT_NAME2: &'static str = "2.5.29.18";
pub const szOID_CRL_REASON_CODE: &'static str = "2.5.29.21";
pub const szOID_REASON_CODE_HOLD: &'static str = "2.5.29.23";
pub const szOID_CRL_DIST_POINTS: &'static str = "2.5.29.31";
pub const szOID_ENHANCED_KEY_USAGE: &'static str = "2.5.29.37";
pub const szOID_ANY_ENHANCED_KEY_USAGE: &'static str = "2.5.29.37.0";
pub const szOID_CRL_NUMBER: &'static str = "2.5.29.20";
pub const szOID_DELTA_CRL_INDICATOR: &'static str = "2.5.29.27";
pub const szOID_ISSUING_DIST_POINT: &'static str = "2.5.29.28";
pub const szOID_FRESHEST_CRL: &'static str = "2.5.29.46";
pub const szOID_NAME_CONSTRAINTS: &'static str = "2.5.29.30";
pub const szOID_POLICY_MAPPINGS: &'static str = "2.5.29.33";
pub const szOID_LEGACY_POLICY_MAPPINGS: &'static str = "2.5.29.5";
pub const szOID_POLICY_CONSTRAINTS: &'static str = "2.5.29.36";
pub const szOID_RENEWAL_CERTIFICATE: &'static str = "1.3.6.1.4.1.311.13.1";
pub const szOID_ENROLLMENT_NAME_VALUE_PAIR: &'static str = "1.3.6.1.4.1.311.13.2.1";
pub const szOID_ENROLLMENT_CSP_PROVIDER: &'static str = "1.3.6.1.4.1.311.13.2.2";
pub const szOID_OS_VERSION: &'static str = "1.3.6.1.4.1.311.13.2.3";
pub const szOID_ENROLLMENT_AGENT: &'static str = "1.3.6.1.4.1.311.20.2.1";
pub const szOID_PKIX: &'static str = "1.3.6.1.5.5.7";
pub const szOID_PKIX_PE: &'static str = "1.3.6.1.5.5.7.1";
pub const szOID_AUTHORITY_INFO_ACCESS: &'static str = "1.3.6.1.5.5.7.1.1";
pub const szOID_SUBJECT_INFO_ACCESS: &'static str = "1.3.6.1.5.5.7.1.11";
pub const szOID_BIOMETRIC_EXT: &'static str = "1.3.6.1.5.5.7.1.2";
pub const szOID_QC_STATEMENTS_EXT: &'static str = "1.3.6.1.5.5.7.1.3";
pub const szOID_LOGOTYPE_EXT: &'static str = "1.3.6.1.5.5.7.1.12";
pub const szOID_CERT_EXTENSIONS: &'static str = "1.3.6.1.4.1.311.2.1.14";
pub const szOID_NEXT_UPDATE_LOCATION: &'static str = "1.3.6.1.4.1.311.10.2";
pub const szOID_REMOVE_CERTIFICATE: &'static str = "1.3.6.1.4.1.311.10.8.1";
pub const szOID_CROSS_CERT_DIST_POINTS: &'static str = "1.3.6.1.4.1.311.10.9.1";
pub const szOID_CTL: &'static str = "1.3.6.1.4.1.311.10.1";
pub const szOID_SORTED_CTL: &'static str = "1.3.6.1.4.1.311.10.1.1";
pub const szOID_SERIALIZED: &'static str = "1.3.6.1.4.1.311.10.3.3.1";
pub const szOID_NT_PRINCIPAL_NAME: &'static str = "1.3.6.1.4.1.311.20.2.3";
pub const szOID_INTERNATIONALIZED_EMAIL_ADDRESS: &'static str = "1.3.6.1.4.1.311.20.2.4";
pub const szOID_PRODUCT_UPDATE: &'static str = "1.3.6.1.4.1.311.31.1";
pub const szOID_ANY_APPLICATION_POLICY: &'static str = "1.3.6.1.4.1.311.10.12.1";
pub const szOID_AUTO_ENROLL_CTL_USAGE: &'static str = "1.3.6.1.4.1.311.20.1";
pub const szOID_ENROLL_CERTTYPE_EXTENSION: &'static str = "1.3.6.1.4.1.311.20.2";
pub const szOID_CERT_MANIFOLD: &'static str = "1.3.6.1.4.1.311.20.3";
pub const szOID_CERTSRV_CA_VERSION: &'static str = "1.3.6.1.4.1.311.21.1";
pub const szOID_CERTSRV_PREVIOUS_CERT_HASH: &'static str = "1.3.6.1.4.1.311.21.2";
pub const szOID_CRL_VIRTUAL_BASE: &'static str = "1.3.6.1.4.1.311.21.3";
pub const szOID_CRL_NEXT_PUBLISH: &'static str = "1.3.6.1.4.1.311.21.4";
pub const szOID_KP_CA_EXCHANGE: &'static str = "1.3.6.1.4.1.311.21.5";
pub const szOID_KP_KEY_RECOVERY_AGENT: &'static str = "1.3.6.1.4.1.311.21.6";
pub const szOID_CERTIFICATE_TEMPLATE: &'static str = "1.3.6.1.4.1.311.21.7";
pub const szOID_ENTERPRISE_OID_ROOT: &'static str = "1.3.6.1.4.1.311.21.8";
pub const szOID_RDN_DUMMY_SIGNER: &'static str = "1.3.6.1.4.1.311.21.9";
pub const szOID_APPLICATION_CERT_POLICIES: &'static str = "1.3.6.1.4.1.311.21.10";
pub const szOID_APPLICATION_POLICY_MAPPINGS: &'static str = "1.3.6.1.4.1.311.21.11";
pub const szOID_APPLICATION_POLICY_CONSTRAINTS: &'static str = "1.3.6.1.4.1.311.21.12";
pub const szOID_ARCHIVED_KEY_ATTR: &'static str = "1.3.6.1.4.1.311.21.13";
pub const szOID_CRL_SELF_CDP: &'static str = "1.3.6.1.4.1.311.21.14";
pub const szOID_REQUIRE_CERT_CHAIN_POLICY: &'static str = "1.3.6.1.4.1.311.21.15";
pub const szOID_ARCHIVED_KEY_CERT_HASH: &'static str = "1.3.6.1.4.1.311.21.16";
pub const szOID_ISSUED_CERT_HASH: &'static str = "1.3.6.1.4.1.311.21.17";
pub const szOID_DS_EMAIL_REPLICATION: &'static str = "1.3.6.1.4.1.311.21.19";
pub const szOID_REQUEST_CLIENT_INFO: &'static str = "1.3.6.1.4.1.311.21.20";
pub const szOID_ENCRYPTED_KEY_HASH: &'static str = "1.3.6.1.4.1.311.21.21";
pub const szOID_CERTSRV_CROSSCA_VERSION: &'static str = "1.3.6.1.4.1.311.21.22";
pub const szOID_NTDS_REPLICATION: &'static str = "1.3.6.1.4.1.311.25.1";
pub const szOID_SUBJECT_DIR_ATTRS: &'static str = "2.5.29.9";
pub const szOID_PKIX_KP: &'static str = "1.3.6.1.5.5.7.3";
pub const szOID_PKIX_KP_SERVER_AUTH: &'static str = "1.3.6.1.5.5.7.3.1";
pub const szOID_PKIX_KP_CLIENT_AUTH: &'static str = "1.3.6.1.5.5.7.3.2";
pub const szOID_PKIX_KP_CODE_SIGNING: &'static str = "1.3.6.1.5.5.7.3.3";
pub const szOID_PKIX_KP_EMAIL_PROTECTION: &'static str = "1.3.6.1.5.5.7.3.4";
pub const szOID_PKIX_KP_IPSEC_END_SYSTEM: &'static str = "1.3.6.1.5.5.7.3.5";
pub const szOID_PKIX_KP_IPSEC_TUNNEL: &'static str = "1.3.6.1.5.5.7.3.6";
pub const szOID_PKIX_KP_IPSEC_USER: &'static str = "1.3.6.1.5.5.7.3.7";
pub const szOID_PKIX_KP_TIMESTAMP_SIGNING: &'static str = "1.3.6.1.5.5.7.3.8";
pub const szOID_PKIX_KP_OCSP_SIGNING: &'static str = "1.3.6.1.5.5.7.3.9";
pub const szOID_PKIX_OCSP_NOCHECK: &'static str = "1.3.6.1.5.5.7.48.1.5";
pub const szOID_PKIX_OCSP_NONCE: &'static str = "1.3.6.1.5.5.7.48.1.2";
pub const szOID_IPSEC_KP_IKE_INTERMEDIATE: &'static str = "1.3.6.1.5.5.8.2.2";
pub const szOID_PKINIT_KP_KDC: &'static str = "1.3.6.1.5.2.3.5";
pub const szOID_KP_CTL_USAGE_SIGNING: &'static str = "1.3.6.1.4.1.311.10.3.1";
pub const szOID_KP_TIME_STAMP_SIGNING: &'static str = "1.3.6.1.4.1.311.10.3.2";
pub const szOID_SERVER_GATED_CRYPTO: &'static str = "1.3.6.1.4.1.311.10.3.3";
pub const szOID_SGC_NETSCAPE: &'static str = "2.16.840.1.113730.4.1";
pub const szOID_KP_EFS: &'static str = "1.3.6.1.4.1.311.10.3.4";
pub const szOID_EFS_RECOVERY: &'static str = "1.3.6.1.4.1.311.10.3.4.1";
pub const szOID_WHQL_CRYPTO: &'static str = "1.3.6.1.4.1.311.10.3.5";
pub const szOID_NT5_CRYPTO: &'static str = "1.3.6.1.4.1.311.10.3.6";
pub const szOID_OEM_WHQL_CRYPTO: &'static str = "1.3.6.1.4.1.311.10.3.7";
pub const szOID_EMBEDDED_NT_CRYPTO: &'static str = "1.3.6.1.4.1.311.10.3.8";
pub const szOID_ROOT_LIST_SIGNER: &'static str = "1.3.6.1.4.1.311.10.3.9";
pub const szOID_KP_QUALIFIED_SUBORDINATION: &'static str = "1.3.6.1.4.1.311.10.3.10";
pub const szOID_KP_KEY_RECOVERY: &'static str = "1.3.6.1.4.1.311.10.3.11";
pub const szOID_KP_DOCUMENT_SIGNING: &'static str = "1.3.6.1.4.1.311.10.3.12";
pub const szOID_KP_LIFETIME_SIGNING: &'static str = "1.3.6.1.4.1.311.10.3.13";
pub const szOID_KP_MOBILE_DEVICE_SOFTWARE: &'static str = "1.3.6.1.4.1.311.10.3.14";
pub const szOID_KP_SMART_DISPLAY: &'static str = "1.3.6.1.4.1.311.10.3.15";
pub const szOID_KP_CSP_SIGNATURE: &'static str = "1.3.6.1.4.1.311.10.3.16";
pub const szOID_DRM: &'static str = "1.3.6.1.4.1.311.10.5.1";
pub const szOID_DRM_INDIVIDUALIZATION: &'static str = "1.3.6.1.4.1.311.10.5.2";
pub const szOID_LICENSES: &'static str = "1.3.6.1.4.1.311.10.6.1";
pub const szOID_LICENSE_SERVER: &'static str = "1.3.6.1.4.1.311.10.6.2";
pub const szOID_KP_SMARTCARD_LOGON: &'static str = "1.3.6.1.4.1.311.20.2.2";
pub const szOID_KP_KERNEL_MODE_CODE_SIGNING: &'static str = "1.3.6.1.4.1.311.61.1.1";
pub const szOID_KP_KERNEL_MODE_TRUSTED_BOOT_SIGNING: &'static str = "1.3.6.1.4.1.311.61.4.1";
pub const szOID_REVOKED_LIST_SIGNER: &'static str = "1.3.6.1.4.1.311.10.3.19";
pub const szOID_WINDOWS_KITS_SIGNER: &'static str = "1.3.6.1.4.1.311.10.3.20";
pub const szOID_WINDOWS_RT_SIGNER: &'static str = "1.3.6.1.4.1.311.10.3.21";
pub const szOID_PROTECTED_PROCESS_LIGHT_SIGNER: &'static str = "1.3.6.1.4.1.311.10.3.22";
pub const szOID_WINDOWS_TCB_SIGNER: &'static str = "1.3.6.1.4.1.311.10.3.23";
pub const szOID_PROTECTED_PROCESS_SIGNER: &'static str = "1.3.6.1.4.1.311.10.3.24";
pub const szOID_WINDOWS_THIRD_PARTY_COMPONENT_SIGNER: &'static str = "1.3.6.1.4.1.311.10.3.25";
pub const szOID_WINDOWS_SOFTWARE_EXTENSION_SIGNER: &'static str = "1.3.6.1.4.1.311.10.3.26";
pub const szOID_DISALLOWED_LIST: &'static str = "1.3.6.1.4.1.311.10.3.30";
pub const szOID_SYNC_ROOT_CTL_EXT: &'static str = "1.3.6.1.4.1.311.10.3.50";
pub const szOID_KP_KERNEL_MODE_HAL_EXTENSION_SIGNING: &'static str = "1.3.6.1.4.1.311.61.5.1";
pub const szOID_WINDOWS_STORE_SIGNER: &'static str = "1.3.6.1.4.1.311.76.3.1";
pub const szOID_DYNAMIC_CODE_GEN_SIGNER: &'static str = "1.3.6.1.4.1.311.76.5.1";
pub const szOID_MICROSOFT_PUBLISHER_SIGNER: &'static str = "1.3.6.1.4.1.311.76.8.1";
pub const szOID_YESNO_TRUST_ATTR: &'static str = "1.3.6.1.4.1.311.10.4.1";
pub const szOID_PKIX_POLICY_QUALIFIER_CPS: &'static str = "1.3.6.1.5.5.7.2.1";
pub const szOID_PKIX_POLICY_QUALIFIER_USERNOTICE: &'static str = "1.3.6.1.5.5.7.2.2";
pub const szOID_ROOT_PROGRAM_FLAGS: &'static str = "1.3.6.1.4.1.311.60.1.1";
//9221
pub type HCERTSTORE = *mut ::c_void;
#[repr(C)] #[derive(Clone, Copy, Debug)]
pub struct CERT_CONTEXT {
    pub dwCertEncodingType: ::DWORD,
    pub pbCertEncoded: *mut ::BYTE,
    pub cbCertEncoded: ::DWORD,
    pub pCertInfo: ::PCERT_INFO,
    pub hCertStore: HCERTSTORE,
}
pub type PCERT_CONTEXT = *mut CERT_CONTEXT;
pub type PCCERT_CONTEXT = *const CERT_CONTEXT;
