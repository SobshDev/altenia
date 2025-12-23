import { z } from 'zod';

export const changeEmailSchema = z.object({
  newEmail: z.string().min(1, 'New email is required').email('Invalid email address'),
  currentPassword: z.string().min(1, 'Current password is required'),
});

export const changePasswordSchema = z
  .object({
    currentPassword: z.string().min(1, 'Current password is required'),
    newPassword: z
      .string()
      .min(8, 'Password must be at least 8 characters')
      .regex(/[A-Z]/, 'Password must contain an uppercase letter')
      .regex(/[a-z]/, 'Password must contain a lowercase letter')
      .regex(/[0-9]/, 'Password must contain a number'),
    confirmNewPassword: z.string().min(1, 'Please confirm your new password'),
  })
  .refine((data) => data.newPassword === data.confirmNewPassword, {
    message: 'Passwords do not match',
    path: ['confirmNewPassword'],
  });

export type ChangeEmailFormValues = z.infer<typeof changeEmailSchema>;
export type ChangePasswordFormValues = z.infer<typeof changePasswordSchema>;
