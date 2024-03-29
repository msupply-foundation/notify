import React, { useEffect, useState } from 'react';
import {
  useTranslation,
  useNotification,
  useParams,
  useBreadcrumbs,
} from '@notify-frontend/common';

import { ScheduledNotificationEditForm } from './ScheduledNotificationEditForm';
import { BaseNotificationEditPage } from '../Base/BaseNotificationEditPage';
import { ScheduledNotification } from '../../types';
import {
  buildScheduledNotificationInputs,
  defaultSchedulerNotification,
  parseScheduledNotificationConfig,
} from './parseConfig';
import { useUpdateNotificationConfig } from '../../api/hooks/useUpdateNotificationConfig';
import {
  NotificationConfigRowFragment,
  useNotificationConfigs,
} from '../../api';
import { validateTemplate } from './tera';

export const ScheduledNotificationEditPage = () => {
  const t = useTranslation('system');
  const { error } = useNotification();
  const parsingErrorSnack = error(t('error.parsing-notification-config'));
  const { setSuffix } = useBreadcrumbs();

  const { id } = useParams<{ id: string }>();
  const [draft, setDraft] = useState<ScheduledNotification>(
    defaultSchedulerNotification
  );

  // Get the notification config from the API
  const { data, isLoading } = useNotificationConfigs({
    filterBy: { id: { equalTo: id } },
  });

  useEffect(() => {
    const entity = data?.nodes[0];
    // Once we get the notification config from the API, parse it and load into the draft
    const parsedDraft = parseScheduledNotificationConfig(
      (entity as NotificationConfigRowFragment) ?? null,
      parsingErrorSnack
    );
    if (parsedDraft) {
      // Backend expects this value to be null if unedited so don't load it from config
      parsedDraft.nextDueDatetime = null;
    }
    setDraft(parsedDraft ?? defaultSchedulerNotification);
    if (parsedDraft?.title) setSuffix(parsedDraft?.title);
  }, [data]);

  const { mutateAsync: update, isLoading: updateIsLoading } =
    useUpdateNotificationConfig();

  const onSave = async (draft: ScheduledNotification) => {
    const inputs = buildScheduledNotificationInputs(draft);
    await update({ input: inputs.update });
  };

  const isValidTemplate = (template: string) => {
    if (!template) return false;
    try {
      validateTemplate(template);
      return true;
    } catch (e) {
      return false;
    }
  };

  const isInvalid =
    !draft.title ||
    !isValidTemplate(draft.subjectTemplate) ||
    !isValidTemplate(draft.bodyTemplate) ||
    // no recipients selected
    (!draft.recipientListIds.length &&
      !draft.recipientIds.length &&
      !draft.sqlRecipientListIds.length);

  return (
    <BaseNotificationEditPage
      isLoading={isLoading || updateIsLoading}
      isInvalid={isInvalid}
      allowParameterSets={true}
      showRunButton={true}
      onSave={onSave}
      draft={draft}
      setDraft={setDraft}
      CustomForm={ScheduledNotificationEditForm}
    />
  );
};
