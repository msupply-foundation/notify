import React, { useMemo, useState } from 'react';
import {
  BasicTextInput,
  Box,
  BufferedTextArea,
  EditIcon,
  FnUtils,
  Grid,
  IconButton,
  KeyedParams,
  LoadingButton,
  SaveIcon,
  TeraUtils,
  Tooltip,
  Typography,
  ZapIcon,
  isValidVariableName,
  useDetailPanel,
  useTranslation,
  validateVariableNameHelperText,
  useNotification,
  AppFooterPortal,
  AppBarContentPortal,
  Paper,
  useToggle,
  Stack,
  useLocalStorage,
} from '@notify-frontend/common';
import { DraftNotificationQuery } from './types';
import { useUpdateNotificationQuery } from '../api';
import { NotificationQueryRowFragment } from '../api/operations.generated';
import { SidePanel } from './SidePanel';

const createNotificationQuery = (
  seed?: DraftNotificationQuery | null
): DraftNotificationQuery => ({
  id: FnUtils.generateUUID(),
  name: '',
  referenceName: '',
  description: '',
  query: '',
  requiredParameters: [],
  ...seed,
});

export const invalidName = (name: string) => {
  const nameIncorrectLength = name.length < 3 || name.length > 75;
  const nameContainsIllegalChars = name.match(/[^ 0-9A-Za-z_\-@.+:/()]/);

  return !name.trim() || !!nameContainsIllegalChars || nameIncorrectLength;
};

type NotificationQueryEditFormProps = {
  entity: NotificationQueryRowFragment | undefined;
  runQuery: (query: string, params: string) => Promise<void>;
  queryLoading: boolean;
  generatedQuery: string;
};

export const QueryEditor = ({
  entity,
  runQuery,
  queryLoading,
  generatedQuery,
}: NotificationQueryEditFormProps) => {
  const t = useTranslation('system');

  const { error } = useNotification();

  const { open: openSidePanel, isOpen } = useDetailPanel();

  const [isSaved, setIsSaved] = useState(true);
  const {
    isOn: isEditingName,
    toggleOn: editNameToggleOn,
    toggleOff: editNameToggleOff,
  } = useToggle(false);

  const [draft, setDraft] = useState(createNotificationQuery(entity));
  const hasParams = useMemo(
    () => TeraUtils.extractParams(draft.query).length > 0,
    [draft]
  );

  const onUpdate = (patch: Partial<DraftNotificationQuery>) => {
    setDraft({ ...draft, ...patch });
    // Update parameters
    if (patch.query) {
      const params = TeraUtils.extractParams(patch.query);
      setDraft({ ...draft, ...patch, requiredParameters: params });
    }
    setIsSaved(false);
  };


  const [userQueryParameters, setUserQueryParameters] = useLocalStorage('/query_parameters');

  const [queryParams, setQueryParams] = useState<KeyedParams>(userQueryParameters ?? {});
  const onUpdateQueryParams = (key: string, value: string) => {
    const patch = { [key]: value };
    setQueryParams({ ...queryParams, ...patch });
  };

  const { mutateAsync: update, isLoading: updateIsLoading } =
    useUpdateNotificationQuery();

  const onSave = async (draft: DraftNotificationQuery) => {
    const { id, name, referenceName, description, query, requiredParameters } =
      draft;
    const input = {
      id,
      name,
      referenceName,
      description,
      query,
      requiredParameters,
    };

    await update({ input });
    editNameToggleOff();
    setIsSaved(true);
  };

  const allParamsSet = TeraUtils.extractParams(draft.query).every(param => {
    return queryParams[param] !== (undefined && "");
  });

  let testQueryButton = (
    <LoadingButton
      variant="outlined"
      disabled={queryLoading || !draft.query || !allParamsSet}
      isLoading={queryLoading}
      startIcon={<ZapIcon />}
      onClick={() => {
        runQuery(draft.query, TeraUtils.keyedParamsAsTeraJson(queryParams));
      }}
    >
      {t('label.test-sql-query')}
    </LoadingButton>
  );
  // If there is a query but we don't have all the parameters yet, replace the test button with an edit params button
  if (draft.query && hasParams && !isOpen && !allParamsSet) {
    testQueryButton = (
      <LoadingButton
        variant="contained"
        disabled={false}
        isLoading={false}
        startIcon={<EditIcon />}
        onClick={openSidePanel}
      >
        {t('label.edit-parameters')}
      </LoadingButton>
    );
  }
  return (
    <>
      <SidePanel
        query={draft.query}
        queryParams={queryParams}
        onUpdateQueryParams={onUpdateQueryParams}
        generatedQuery={generatedQuery}
        userQueryParameters={userQueryParameters}
        setUserQueryParameters={setUserQueryParameters}
      />
      {/* Description/Details section */}
      <AppBarContentPortal sx={{ paddingBottom: '16px', flex: 1 }}>
        <Paper
          sx={{
            borderRadius: '16px',
            boxShadow: theme => theme.shadows[1],
            padding: '21px',
            height: 'fit-content',
            backgroundColor: 'background',
          }}
        >
          {isEditingName ? (
            <Stack>
              <BasicTextInput
                required
                value={draft.name}
                error={invalidName(draft.name)}
                helperText={
                  invalidName(draft.name)
                    ? t('helper-text.recipient-list-name')
                    : null
                }
                onChange={e => onUpdate({ name: e.target.value })}
                label={t('label.name')}
                InputLabelProps={{ shrink: true }}
              />
              <BasicTextInput
                required
                value={draft.referenceName}
                error={!isValidVariableName(draft.referenceName)}
                helperText={
                  <Tooltip title={t('helper-text.reference_name')}>
                    <span>
                      {validateVariableNameHelperText(draft.referenceName, t) ??
                        t('helper-text.reference_name')}
                    </span>
                  </Tooltip>
                }
                onChange={e => onUpdate({ referenceName: e.target.value })}
                label={t('label.reference-name')}
                InputLabelProps={{ shrink: true }}
              />

              <BufferedTextArea
                value={draft.description}
                onChange={e => onUpdate({ description: e.target.value })}
                label={t('label.description')}
                InputProps={{ sx: { backgroundColor: 'background.menu' } }}
                InputLabelProps={{ shrink: true }}
                rows={4}
              />
            </Stack>
          ) : (
            <Typography
              sx={{
                fontSize: '18px',
                fontWeight: 'bold',
                color: 'gray.dark',
              }}
              onClick={editNameToggleOn}
            >
              {draft?.name} ({draft?.referenceName})
              <IconButton
                onClick={editNameToggleOn}
                icon={<EditIcon />}
                label={t('label.edit')}
              />
            </Typography>
          )}

          <Grid item xs={12}>
            <BufferedTextArea
              value={draft.query}
              onChange={e => onUpdate({ query: e.target.value })}
              label={t('label.query')}
              InputProps={{
                sx: {
                  backgroundColor: 'background.menu',
                  textarea: {
                    resize: 'vertical',
                  },
                },
              }}
              InputLabelProps={{ shrink: true }}
              helperText={t('helper-text.sql-query')}
              minRows={4}
              maxRows={10}
            />
          </Grid>
        </Paper>
      </AppBarContentPortal>

      <AppFooterPortal
        Content={
          <Box
            gap={2}
            display="flex"
            flexDirection="row"
            alignItems="center"
            height={64}
          >
            <Box flex={1} display="flex" justifyContent="flex-start" gap={1}>
              <Typography
                component={'span'}
                sx={{ fontWeight: 'bold', color: 'gray.dark' }}
              >
                {t('label.parameters')}:
              </Typography>
              {!hasParams ? (
                <Typography component={'span'} sx={{ color: 'gray.light' }}>
                  {t('message.no-parameters')}
                </Typography>
              ) : (
                <>
                  <Typography component={'span'} sx={{ color: 'gray.dark' }}>
                    {TeraUtils.extractParams(draft.query).join(', ')}
                  </Typography>
                  <IconButton
                    height="24px"
                    width="24px"
                    onClick={openSidePanel}
                    icon={<EditIcon />}
                    label={t('label.edit')}
                  />
                </>
              )}
            </Box>
            <Box flex={1} display="flex" justifyContent="flex-end" gap={2}>
              {testQueryButton}
              <LoadingButton
                startIcon={<SaveIcon />}
                onClick={() => {
                  onSave(draft).catch(err => {
                    console.error(err);
                    error(err)();
                  });
                }}
                disabled={isSaved || invalidName(draft.name)}
                isLoading={updateIsLoading}
                sx={{ marginRight: 1 }}
              >
                {t('button.save')}
              </LoadingButton>
            </Box>
          </Box>
        }
      />
    </>
  );
};
