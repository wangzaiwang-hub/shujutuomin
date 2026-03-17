import { invoke } from "@tauri-apps/api/core";
import type {
  MaskFileOptions,
  MaskResult,
  PreviewOptions,
  PreviewResult,
  EncryptOptions,
  DecryptOptions,
  SandboxFile,
  MaskRule,
  BatchJobOptions,
  BatchStatus,
} from "@/types/commands";
import type { LogEntry, ProcessingHistory, UserSetting, DatabaseStatistics } from "@/types/log";

export const tauriCommands = {
  // Masking
  maskFile: (options: MaskFileOptions) =>
    invoke<MaskResult>("mask_file", { options }),

  previewMasking: (options: PreviewOptions) =>
    invoke<PreviewResult>("preview_masking", { options }),

  // Crypto
  generatePassphrase: () =>
    invoke<string>("generate_passphrase"),

  encryptMapping: (options: EncryptOptions) =>
    invoke<string>("encrypt_mapping", { options }),

  decryptMapping: (options: DecryptOptions) =>
    invoke<string>("decrypt_mapping", { options }),

  // Sandbox
  verifyPin: (pin: string) =>
    invoke<boolean>("verify_pin", { pin }),

  setPin: (pin: string) =>
    invoke<void>("set_pin", { pin }),

  listSandboxFiles: () =>
    invoke<SandboxFile[]>("list_sandbox_files"),

  listFilesInDirectory: (directory: string) =>
    invoke<SandboxFile[]>("list_files_in_directory", { directory }),

  exportSandbox: (fileName: string, destPath: string, passphrase: string) =>
    invoke<void>("export_sandbox", { fileName, destPath, passphrase }),

  importSandbox: (srcPath: string, passphrase: string) =>
    invoke<SandboxFile>("import_sandbox", { srcPath, passphrase }),

  // Rules
  getRules: () =>
    invoke<MaskRule[]>("get_rules"),

  saveRules: (rules: MaskRule[]) =>
    invoke<void>("save_rules", { rules }),

  // Batch
  startBatchJob: (options: BatchJobOptions) =>
    invoke<string>("start_batch_job", { options }),

  getBatchStatus: (jobId: string) =>
    invoke<BatchStatus>("get_batch_status", { jobId }),

  cancelBatchJob: (jobId: string) =>
    invoke<void>("cancel_batch_job", { jobId }),

  // Database - Logs
  initializeDatabase: () =>
    invoke<void>("initialize_database"),

  addLogEntry: (level: string, message: string, details?: string, filePath?: string, operationType?: string) =>
    invoke<void>("add_log_entry", { 
      request: { level, message, details, file_path: filePath, operation_type: operationType }
    }),

  getLogs: (limit?: number, offset?: number, levelFilter?: string) =>
    invoke<LogEntry[]>("get_logs", { 
      params: { limit, offset, level_filter: levelFilter }
    }),

  getLogsCount: (levelFilter?: string) =>
    invoke<number>("get_logs_count", { level_filter: levelFilter }),

  clearAllLogs: () =>
    invoke<void>("clear_all_logs"),

  cleanupOldLogs: (days: number) =>
    invoke<number>("cleanup_old_logs", { days }),

  // Database - User Settings
  saveUserSetting: (key: string, value: string) =>
    invoke<void>("save_user_setting", { key, value }),

  getUserSetting: (key: string) =>
    invoke<string | null>("get_user_setting", { key }),

  getAllUserSettings: () =>
    invoke<UserSetting[]>("get_all_user_settings"),

  deleteUserSetting: (key: string) =>
    invoke<void>("delete_user_setting", { key }),

  // Database - Processing History
  addProcessingHistory: (
    filePath: string,
    outputPath: string,
    ruleIds: string[],
    fileSize: number,
    maskedCount: number,
    processingTimeMs: number,
    status: string,
    errorMessage?: string
  ) =>
    invoke<void>("add_processing_history", {
      request: {
        file_path: filePath,
        output_path: outputPath,
        rule_ids: ruleIds,
        file_size: fileSize,
        masked_count: maskedCount,
        processing_time_ms: processingTimeMs,
        status,
        error_message: errorMessage,
      }
    }),

  getProcessingHistory: (limit?: number, offset?: number) =>
    invoke<ProcessingHistory[]>("get_processing_history", { limit, offset }),

  getStatistics: () =>
    invoke<DatabaseStatistics>("get_statistics"),

  getDatabaseInfo: () =>
    invoke<any>("get_database_info"),

  // Database Migration
  migrateOldDatabase: () =>
    invoke<string>("migrate_old_database"),

  // Proxy
  fetchWebpage: (url: string) =>
    invoke<{content: string, status: number, contentType: string}>("fetch_webpage", { url }),
};
