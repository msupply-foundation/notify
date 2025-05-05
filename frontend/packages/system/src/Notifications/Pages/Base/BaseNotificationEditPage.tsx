import React, { useState } from 'react';
import Alert from '@mui/material/Alert';
import AlertTitle from '@mui/material/AlertTitle';
import {
  Grid,
  useTranslation,
  LoadingButton,
  InlineSpinner,
  AppBarContentPortal,
  Box,
  AppFooterPortal,
  ButtonWithIcon,
  CloseIcon,
  CopyIcon,
  useBreadcrumbs,
  SaveIcon,
  ConfigStatus,
  FormLabel,
  Switch,
  useDetailPanel,
  AppBarButtonsPortal,
  KeyedParams,
  FnUtils,
  useNavigate,
  useConfirmationModal,
  RunIcon,
  RouteBuilder,
  BaseButton,
  ListIcon,
  Tooltip,
} from '@notify-frontend/common';

import { BaseNotificationConfig } from '../../types';
import { BaseNotificationAppBar } from './BaseNotificationAppBar';
import { NotificationDetailPanel } from './NotificationDetailPanel';
import {
  useRecipientLists,
  useRecipients,
  useSqlRecipientLists,
} from 'packages/system/src/Recipients/api';
import { useDuplicateNotificationConfig } from '../../api/hooks/useDuplicateNotificationConfig';
import { configRoute } from '../../navigate';
import { AppRoute } from 'packages/config/src';

interface BaseNotificationEditPageProps<T extends BaseNotificationConfig> {
  isInvalid: boolean;
  isLoading: boolean;
  allowParameterSets?: boolean;
  showRunButton?: boolean;
  draft: T;
  setDraft: (draft: T) => void;
  onSave: (draft: T) => Promise<void>;
  CustomForm: React.FC<{
    onUpdate: (patch: Partial<T>) => void;
    draft: T;
  }>;
}

export const BaseNotificationEditPage = <T extends BaseNotificationConfig>({
  isInvalid,
  isLoading,
  allowParameterSets = false,
  showRunButton = false,
  draft,
  setDraft,
  onSave,
  CustomForm,
}: BaseNotificationEditPageProps<T>) => {
  const t = useTranslation(['system']);
  const { OpenButton } = useDetailPanel(t('label.parameters'));
  const { navigateUpOne } = useBreadcrumbs();
  const [errorMessage, setErrorMessage] = useState('');
  const [isSaved, setIsSaved] = useState(true);
  const navigate = useNavigate();

  const isEnabled = (status: ConfigStatus) => {
    return status == ConfigStatus.Enabled;
  };

  // TODO: https://github.com/msupply-foundation/notify/issues/238 handle pagination
  const { data: recipients } = useRecipients({ first: 1000 });
  const { data: recipientLists } = useRecipientLists({ first: 1000 });
  const { data: sqlRecipientLists } = useSqlRecipientLists({ first: 1000 });

  const { mutateAsync: duplicate } = useDuplicateNotificationConfig();

  const onUpdate = (patch: Partial<T>) => {
    setDraft({ ...draft, ...patch });
    setIsSaved(false);
  };

  const onDuplicate = async () => {
    const newId = FnUtils.generateUUID();
    await duplicate({ input: { oldId: draft.id, newId } });
    navigate(configRoute(draft.kind, newId));
  };

  const showDuplicateConfirmation = useConfirmationModal({
    onConfirm: onDuplicate,
    message: t('messages.confirm-duplicate'),
    title: t('heading.are-you-sure'),
  });

  const onUpdateParams = (idx: number = 0, key: string, value: string) => {
    const updatedParam = { [key]: value } as KeyedParams;

    const parseParams = draft.parsedParameters;
    if (idx >= draft.parsedParameters.length) {
      parseParams.push(updatedParam);
    } else {
      parseParams[idx] = { ...draft.parsedParameters[idx], ...updatedParam };
    }
    onUpdate({
      ...draft,
      parsedParameters: parseParams,
    });
  };

  const onDeleteParam = (idx: number, key: string | null) => {
    const updatedParams = draft.parsedParameters;
    if (
      updatedParams.length == 0 ||
      idx > updatedParams.length ||
      idx < 0 ||
      updatedParams[idx] == undefined
    ) {
      return;
    }

    if (key == null) {
      updatedParams.splice(idx, 1); // Delete everything for that index
    } else {
      delete updatedParams[idx]![key];
    }

    onUpdate({
      ...draft,
      parsedParameters: updatedParams,
    });
  };

  const requiredRecipientParams = (sqlRecipientLists?.nodes ?? [])
    .filter(list => draft.sqlRecipientListIds.includes(list.id))
    .map(list => list.parameters)
    .flat(1);

  const requiredConfigParams = draft.requiredParameters;

  const requiredParams = requiredRecipientParams.concat(requiredConfigParams);

  const allParamsSet = requiredParams.every(param => {
    if (param) {
      if (!Array.isArray(draft.parsedParameters)) {
        draft.parsedParameters = [draft.parsedParameters];
        draft.parameters = JSON.stringify(draft.parsedParameters);
      }

      return draft.parsedParameters.every(obj => obj[param] !== undefined); // This allows the user to set the param to an empty string if they edit the field then delete the value
    } else {
      return false;
    }
  });

  const saveConfig = () => {
    onSave(draft);
    setIsSaved(true);
  };

  const confirmBeforeSaving = useConfirmationModal({
    title: t('label.notification-is-disabled'),
    message: t('messages.saving-disabled-valid-notification'),
    onConfirm: saveConfig,
  });

  return (
    <>
      {isLoading ? (
        <InlineSpinner />
      ) : (
        <>
          <NotificationDetailPanel
            requiredParams={requiredParams}
            params={draft.parsedParameters}
            allowParameterSets={allowParameterSets}
            onUpdateParams={onUpdateParams}
            onDeleteParam={onDeleteParam}
            onChangeParameterQuery={x => onUpdate({ parameterQueryId: x } as Partial<T>)}
            parameterQueryId={draft.parameterQueryId}
          />
          <AppBarButtonsPortal sx={{ display: 'flex', gap: '14px' }}>
            <BaseButton
              onClick={() => {
                navigate(
                  `${RouteBuilder.create(
                    AppRoute.NotificationEvents
                  ).build()}?notificationConfigId=${draft.id}`
                );
              }}
              variant="outlined"
              startIcon={<ListIcon />}
            >
              {t('button.view-recent-events')}
            </BaseButton>
            {OpenButton}
          </AppBarButtonsPortal>
          <AppBarContentPortal sx={{ paddingBottom: '16px', flex: 1 }}>
            <BaseNotificationAppBar
              draft={draft}
              onUpdate={onUpdate}
              recipientLists={recipientLists?.nodes ?? []}
              recipients={recipients?.nodes ?? []}
              sqlRecipientLists={sqlRecipientLists?.nodes ?? []}
            />
          </AppBarContentPortal>
          <Grid flexDirection="column" display="flex" gap={2} width={'100%'}>
            <Box sx={{ paddingLeft: '10px' }}>
              <CustomForm draft={draft} onUpdate={onUpdate} />
              {errorMessage ? (
                <Grid item>
                  <Alert
                    severity="error"
                    onClose={() => {
                      setErrorMessage('');
                    }}
                  >
                    <AlertTitle>{t('error')}</AlertTitle>
                    {errorMessage}
                  </Alert>
                </Grid>
              ) : null}
            </Box>
          </Grid>
          <AppFooterPortal
            Content={
              <Box
                gap={2}
                display="flex"
                flexDirection="row"
                alignItems="center"
                height={64}
              >
                <Box
                  flex={1}
                  display="flex"
                  justifyContent="flex-start"
                  gap={1}
                >
                  <FormLabel
                    sx={{
                      alignSelf: 'flex-start',
                      alignItems: 'center',
                      paddingTop: 1,
                    }}
                  >
                    {t('label.enable')}
                  </FormLabel>
                  <Switch
                    checked={isEnabled(draft.status)}
                    onChange={() => {
                      if (isEnabled(draft.status)) {
                        onUpdate({
                          status: ConfigStatus.Disabled,
                        } as Partial<T>);
                      } else {
                        onUpdate({
                          status: ConfigStatus.Enabled,
                        } as Partial<T>);
                      }
                    }}
                  />
                </Box>
                <Box flex={1} display="flex" justifyContent="flex-end" gap={2}>
                  <ButtonWithIcon
                    shrinkThreshold="lg"
                    Icon={<CloseIcon />}
                    label={t('button.close')}
                    color="secondary"
                    sx={{ fontSize: '12px' }}
                    onClick={navigateUpOne}
                  />

                  <ButtonWithIcon
                    shrinkThreshold="lg"
                    Icon={<CopyIcon />}
                    label={t('button.duplicate')}
                    color="secondary"
                    sx={{ fontSize: '12px' }}
                    onClick={() => {
                      showDuplicateConfirmation();
                    }}
                  />
                  {showRunButton && (
                    <LoadingButton
                      disabled={
                        isInvalid ||
                        !allParamsSet ||
                        draft.status == ConfigStatus.Disabled
                      }
                      isLoading={isLoading}
                      onClick={() => {
                        // Note, this doesn't update state, but that's good we don't want to save the nextDueDatetime again if the save button is used next.
                        draft.nextDueDatetime = new Date().toISOString();
                        saveConfig();
                      }}
                      startIcon={<RunIcon />}
                      sx={{ fontSize: '12px' }}
                    >
                      {isSaved ? t('button.run') : t('button.save-and-run')}
                    </LoadingButton>
                  )}

                  <Tooltip
                    title={
                      isEnabled(draft.status) && isInvalid
                        ? t('messages.saving-enabled-invalid-notification')
                        : ''
                    }
                  >
                    <span>
                      <LoadingButton
                        disabled={
                          isSaved ||
                          (isEnabled(draft.status) && isInvalid) ||
                          !allParamsSet
                        }
                        isLoading={isLoading}
                        onClick={() => {
                          // prompt user if saving a valid config with disabled status - they may have forgotten to enable
                          if (!isInvalid && !isEnabled(draft.status)) {
                            confirmBeforeSaving();
                          } else {
                            saveConfig();
                          }
                        }}
                        startIcon={<SaveIcon />}
                        sx={{ fontSize: '12px' }}
                      >
                        {t('button.save')}
                      </LoadingButton>
                    </span>
                  </Tooltip>
                </Box>
              </Box>
            }
          />
        </>
      )}
    </>
  );
};
