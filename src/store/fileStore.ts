import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { QueuedFile } from "@/types/file";

interface FileStore {
  files: QueuedFile[];
  activeJobId: string | null;
  passphrase: string;
  outputDir: string;
  addFiles: (files: QueuedFile[]) => void;
  removeFile: (id: string) => void;
  updateFile: (id: string, updates: Partial<QueuedFile>) => void;
  clearCompleted: () => void;
  setActiveJob: (id: string | null) => void;
  setPassphrase: (p: string) => void;
  setOutputDir: (dir: string) => void;
}

export const useFileStore = create<FileStore>()(
  persist(
    (set) => ({
      files: [],
      activeJobId: null,
      passphrase: "",
      outputDir: "",

      addFiles: (newFiles) =>
        set((state) => ({
          files: [
            ...state.files,
            ...newFiles.filter(
              (f) => !state.files.some((e) => e.path === f.path)
            ),
          ],
        })),

      removeFile: (id) =>
        set((state) => ({ files: state.files.filter((f) => f.id !== id) })),

      updateFile: (id, updates) =>
        set((state) => ({
          files: state.files.map((f) => (f.id === id ? { ...f, ...updates } : f)),
        })),

      clearCompleted: () =>
        set((state) => ({
          files: state.files.filter((f) => f.status !== "completed"),
        })),

      setActiveJob: (id) => set({ activeJobId: id }),
      setPassphrase: (p) => set({ passphrase: p }),
      setOutputDir: (dir) => set({ outputDir: dir }),
    }),
    {
      name: "file-store",
      // 只持久化需要保存的字段
      partialize: (state) => ({
        passphrase: state.passphrase,
        outputDir: state.outputDir,
      }),
    }
  )
);
