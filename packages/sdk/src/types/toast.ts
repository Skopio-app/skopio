import z from "zod";

export const toastSchema = z.object({
  message: z.string(),
  description: z.string().optional(),
  position: z
    .enum([
      "top-left",
      "top-right",
      "bottom-left",
      "bottom-right",
      "top-center",
      "bottom-center",
    ])
    .optional(),
  duration: z.number().optional(),
  closeButton: z.boolean().optional(),
});
