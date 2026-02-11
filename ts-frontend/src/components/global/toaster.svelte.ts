const newToaster = () => {
  let toasts = $state([] as Toast[]);

  return {
    get toasts() {
      return toasts;
    },
    add(newToastParams: ToastParams) {
      const id = crypto.randomUUID();

      const newToast = { ...defaults, id, ...newToastParams };

      toasts = [...toasts, newToast];
      setTimeout(() => {
        this.remove(id);
      }, newToast.timeout);
    },
    remove(id: string) {
      toasts = toasts.filter((t) => t.id !== id);
    },
  };
};

export const globalToaster = newToaster();

const defaults = {
  type: "info",
  isDismissable: true,
  timeout: 2500,
} as const;

type Toast = {
  id: string;
  type: ToastType;
  isDismissable: boolean;
  message: string;
  timeout: number;
};

export type ToastType = "success" | "failure" | "info";

type ToastParams = {
  message: string;
  type?: ToastType;
  isDismissable?: boolean;
  timeout?: number;
};
