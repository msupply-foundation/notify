import format from 'date-fns/format';
import isValid from 'date-fns/isValid';
import { LocaleKey } from '../..';

export const Formatter = {
  naiveDate: (date?: Date | null): string | null => {
    if (date && isValid(date)) return format(date, 'yyyy-MM-dd');
    else return null;
  },
  // use toISOstring() for date time. Server always expects UTC string.
  expiryDate: (date?: Date | null): string | null => {
    if (date && isValid(date)) return format(date, 'MM/yyyy');
    else return null;
  },
  expiryDateString: (date?: string | null | undefined): string => {
    const expiryDate = date ? Formatter.expiryDate(new Date(date)) : null;
    return expiryDate ?? '';
  },
  logTypeTranslation: (logType: string): LocaleKey =>
    `log.${logType.toLowerCase().replace(/_/g, '-')}` as LocaleKey,
};
