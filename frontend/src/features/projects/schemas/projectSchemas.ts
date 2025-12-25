import { z } from 'zod';

export const createProjectSchema = z.object({
  name: z
    .string()
    .min(1, 'Project name is required')
    .max(100, 'Name must be less than 100 characters'),
  description: z
    .string()
    .max(500, 'Description must be less than 500 characters')
    .optional(),
});

export const updateProjectSchema = z.object({
  name: z
    .string()
    .min(1, 'Project name is required')
    .max(100, 'Name must be less than 100 characters'),
  description: z
    .string()
    .max(500, 'Description must be less than 500 characters')
    .optional(),
});

export const retentionSchema = z.object({
  retention_days: z
    .number()
    .min(1, 'Must be at least 1 day')
    .max(365, 'Cannot exceed 365 days'),
  metrics_retention_days: z
    .number()
    .min(1, 'Must be at least 1 day')
    .max(365, 'Cannot exceed 365 days'),
  traces_retention_days: z
    .number()
    .min(1, 'Must be at least 1 day')
    .max(90, 'Cannot exceed 90 days'),
});

export const createApiKeySchema = z.object({
  name: z
    .string()
    .min(1, 'API key name is required')
    .max(100, 'Name must be less than 100 characters'),
  expires_in_days: z.number().min(1).max(365).optional(),
});

export type CreateProjectFormValues = z.infer<typeof createProjectSchema>;
export type UpdateProjectFormValues = z.infer<typeof updateProjectSchema>;
export type RetentionFormValues = z.infer<typeof retentionSchema>;
export type CreateApiKeyFormValues = z.infer<typeof createApiKeySchema>;
