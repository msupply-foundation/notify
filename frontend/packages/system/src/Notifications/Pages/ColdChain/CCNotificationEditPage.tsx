import React, { useEffect, useState } from 'react';
import {
  FnUtils,
  ConfigKind,
  useTranslation,
  useNotification,
  ConfigStatus,
  useParams,
} from '@notify-frontend/common';
import { CCNotificationEditForm } from './CCNotificationEditForm';
import { BaseNotificationEditPage } from '../Base/BaseNotificationEditPage';
import { CCNotification, ReminderUnits } from '../../types';
import {
  buildColdChainNotificationInputs,
  parseColdChainNotificationConfig,
} from './parseConfig';
import { useUpdateNotificationConfig } from '../../api/hooks/useUpdateNotificationConfig';
import {
  NotificationConfigRowFragment,
  useNotificationConfigs,
} from '../../api';

const createCCNotification = (
  seed: CCNotification | null | undefined
): CCNotification => ({
  id: seed?.id ?? FnUtils.generateUUID(),
  title: seed?.title ?? '',
  kind: seed?.kind ?? ConfigKind.ColdChain,
  nextDueDatetime: null, // We always want this to be null, cold chain doesn't use this field anyway
  highTemp: seed?.highTemp ?? true,
  highTempThreshold: seed?.highTempThreshold ?? 8,
  lowTemp: seed?.lowTemp ?? true,
  lowTempThreshold: seed?.lowTempThreshold ?? 2,
  confirmOk: seed?.confirmOk ?? true,
  noData: seed?.noData ?? true,
  noDataInterval: seed?.noDataInterval ?? 4,
  noDataIntervalUnits: seed?.noDataIntervalUnits ?? ReminderUnits.HOURS,
  remind: seed?.remind ?? true,
  reminderInterval: seed?.reminderInterval ?? 2,
  reminderUnits: seed?.reminderUnits ?? ReminderUnits.HOURS,
  locationIds: seed?.locationIds ?? [],
  sensorIds: seed?.sensorIds ?? [],
  recipientIds: seed?.recipientIds ?? [],
  recipientListIds: seed?.recipientListIds ?? [],
  status: seed?.status ?? ConfigStatus.Disabled,
  sqlRecipientListIds: seed?.sqlRecipientListIds ?? [],
  parameters: seed?.parameters ?? '[{}]',
  parsedParameters: seed?.parsedParameters ?? [],
  requiredParameters: seed?.requiredParameters ?? [],
});

export const CCNotificationEditPage = () => {
  const t = useTranslation('system');
  const { error } = useNotification();
  const parsingErrorSnack = error(t('error.parsing-notification-config'));

  const { id } = useParams<{ id: string }>();

  // Create an empty draft
  const [draft, setDraft] = useState<CCNotification>(() =>
    createCCNotification(null)
  );

  // Get the notification config from the API
  const { data, isLoading } = useNotificationConfigs({
    filterBy: { id: { equalTo: id } },
  });

  useEffect(() => {
    const entity = data?.nodes[0];
    // Once we get the notification config from the API, parse it and load into the draft
    const parsedDraft = parseColdChainNotificationConfig(
      (entity as NotificationConfigRowFragment) ?? null,
      parsingErrorSnack
    );
    setDraft(createCCNotification(parsedDraft));
  }, [data]);

  const { mutateAsync: update, isLoading: updateIsLoading } =
    useUpdateNotificationConfig();

  const onSave = async (draft: CCNotification) => {
    const inputs = buildColdChainNotificationInputs(draft);
    await update({ input: inputs.update });
  };

  const isInvalid =
    !draft.title ||
    // nothing selected
    (!draft.confirmOk &&
      !draft.highTemp &&
      !draft.lowTemp &&
      draft.remind &&
      draft.noData) ||
    // no sensor selected
    !draft.sensorIds.length ||
    // no recipients selected
    (!draft.recipientListIds.length &&
      !draft.recipientIds.length &&
      !draft.sqlRecipientListIds.length);

  return (
    <BaseNotificationEditPage
      isLoading={isLoading || updateIsLoading}
      isInvalid={isInvalid}
      onSave={onSave}
      draft={draft}
      setDraft={setDraft}
      CustomForm={CCNotificationEditForm}
    />
  );
};
