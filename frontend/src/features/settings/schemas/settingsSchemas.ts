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

export const changeOrgNameSchema = z.object({
  name: z.string().min(1, 'Name is required').max(100, 'Name must be less than 100 characters'),
});

export const addMemberSchema = z.object({
  email: z.string().min(1, 'Email is required').email('Invalid email address'),
  role: z.enum(['admin', 'member']),
});

export const changeDisplayNameSchema = z.object({
  displayName: z
    .string()
    .min(1, 'Display name is required')
    .max(30, 'Display name must not exceed 30 characters')
    .regex(
      /^[\p{L}\s\-']+$/u,
      'Display name can only contain letters, spaces, dashes, and apostrophes'
    ),
});

export type ChangeEmailFormValues = z.infer<typeof changeEmailSchema>;
export type ChangePasswordFormValues = z.infer<typeof changePasswordSchema>;
export type ChangeOrgNameFormValues = z.infer<typeof changeOrgNameSchema>;
export type AddMemberFormValues = z.infer<typeof addMemberSchema>;
export type ChangeDisplayNameFormValues = z.infer<typeof changeDisplayNameSchema>;
