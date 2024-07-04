import {
  ConfigKind,
  ConfigStatus,
  CreateNotificationConfigInput,
  UpdateNotificationConfigInput,
} from '@common/types';
import { NotificationConfigRowFragment } from '../../api';
import { ScheduledNotification } from '../../types';
import { FnUtils, TeraUtils } from '@common/utils';

export function parseScheduledNotificationConfig(
  config: NotificationConfigRowFragment | null,
  showError: () => void
): ScheduledNotification | null {
  if (!config) return null;
  try {
    const { configurationData, ...rest } = config;

    return {
      ...defaultSchedulerNotification,
      ...JSON.parse(configurationData),
      ...rest,
    };
  } catch (e) {
    showError();
    // There's not much the user can do, except contact support or input the data again
    // and hope it saves in such a way that we can parse it next time!
    // The missing fields will be populated by default values in the edit modal, but we'll return
    // the base NotificationConfig data that is still usable:
    return {
      ...defaultSchedulerNotification,
      id: config.id,
      title: config.title,
      kind: config.kind,
    };
  }
}

export const defaultSchedulerNotification: ScheduledNotification = {
  id: FnUtils.generateUUID(),
  title: '',
  kind: ConfigKind.Scheduled,
  nextDueDatetime: null, // We always want this to be null, unless triggering a 'run now' action
  recipientListIds: [],
  recipientIds: [],
  sqlRecipientListIds: [],
  parameters: '[]',
  parameterQueryId: null,
  parsedParameters: [],
  requiredParameters: [],
  scheduleFrequency: 'daily',
  scheduleStartTime: new Date(),
  subjectTemplate: '',
  bodyTemplate: '',
  conditional: false,
  conditionTemplate: '',
  notificationQueryIds: [],
  status: ConfigStatus.Disabled,
};

export function buildScheduledNotificationInputs(
  config: ScheduledNotification
): {
  create: CreateNotificationConfigInput;
  update: UpdateNotificationConfigInput;
} {
  const params = [];
  if (!Array.isArray(config.parsedParameters)) {
    config.parsedParameters = [config.parsedParameters];
  }
  for (const param of config.parsedParameters) {
    params.push(TeraUtils.keyedParamsAsTeraParams(param));
  }

  const input = {
    id: config.id,
    title: config.title,
    configurationData: JSON.stringify(config),
    status: config.status,
    parameters: JSON.stringify(params),
    parameterQueryId: config.parameterQueryId,
    recipientIds: config.recipientIds,
    recipientListIds: config.recipientListIds,
    sqlRecipientListIds: config.sqlRecipientListIds,
    nextDueDatetime: config.nextDueDatetime,
  };
  return {
    create: { ...input, kind: config.kind },
    update: input,
  };
}
