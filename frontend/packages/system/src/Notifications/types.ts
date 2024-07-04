import { NotificationConfigRowFragment } from './api';
import { KeyedParams } from '@common/utils';
import { LocaleKey, TypedTFunction } from '@common/intl';

export enum ReminderUnits {
  MINUTES = 'minutes',
  HOURS = 'hours',
}

export function getReminderUnitsFromString(str: string): ReminderUnits {
  switch (str) {
    case 'minutes':
      return ReminderUnits.MINUTES;
    case 'hours':
      return ReminderUnits.HOURS;
    default:
      throw new Error(`Invalid reminder units: ${str}`);
  }
}

export function getReminderUnitsAsOptions(
  t: TypedTFunction<LocaleKey>,
  count?: number
): {
  label: string;
  value: string;
}[] {
  return [
    {
      label: t('label.minutes', { count: count }),
      value: ReminderUnits.MINUTES,
    },
    { label: t('label.hours', { count: count }), value: ReminderUnits.HOURS },
  ];
}

type BaseConfig = Pick<
  NotificationConfigRowFragment,
  | 'id'
  | 'kind'
  | 'title'
  | 'status'
  | 'parameters'
  | 'parameterQueryId'
  | 'recipientIds'
  | 'recipientListIds'
  | 'sqlRecipientListIds'
>;

export interface BaseNotificationConfig extends BaseConfig {
  parsedParameters: KeyedParams[];
  requiredParameters: string[];
  nextDueDatetime: string | null; // Date string in ISO format
}

export interface CCNotification extends BaseNotificationConfig {
  highTemp: boolean;
  highTempThreshold: number;
  lowTemp: boolean;
  lowTempThreshold: number;
  confirmOk: boolean;
  noData: boolean;
  noDataInterval: number;
  noDataIntervalUnits: ReminderUnits;
  remind: boolean;
  reminderInterval: number;
  reminderUnits: ReminderUnits;
  locationIds: string[];
  sensorIds: string[];
}

export interface ScheduledNotification extends BaseNotificationConfig {
  scheduleFrequency: string;
  scheduleStartTime: Date;
  subjectTemplate: string;
  bodyTemplate: string;
  conditional: boolean;
  conditionTemplate: string;
  notificationQueryIds: string[];
}
