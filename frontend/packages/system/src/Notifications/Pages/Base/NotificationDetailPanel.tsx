import React from 'react';
import { CopyIcon, DeleteIcon, DetailPanelPortal, SaveIcon, PlusCircleIcon } from '@common/ui';
import Alert from '@mui/material/Alert';
import AlertTitle from '@mui/material/AlertTitle';
import {
  BufferedTextArea,
  DetailPanelSection,
  IconButton,
  Autocomplete,
} from '@common/components';
import { KeyedParams } from '@common/utils';
import { useQueryParamsState } from '@common/hooks';

import { useNotificationQueries } from '../../../Queries/api';
import { ParameterEditor } from './ParameterEditor';
import { useTranslation } from '@common/intl';

export interface ParamsPanelProps {
  requiredParams: string[];
  params: KeyedParams[];
  allowParameterSets?: boolean;
  onUpdateParams: (idx: number, key: string, value: string) => void;
  onDeleteParam: (idx: number, key: string | null) => void; // Warning: null deletes everything for that index
  onChangeParameterQuery?: (id: string | null) => void;
  parameterQueryId?: string | null;
}

export const NotificationDetailPanel = ({
  requiredParams,
  params,
  allowParameterSets = false,
  onUpdateParams,
  onDeleteParam,
  onChangeParameterQuery = () => {},
  parameterQueryId = null,
}: ParamsPanelProps) => {
  const t = useTranslation('system');

  const [isJsonEdited, setIsJsonEdited] = React.useState(false);
  const [paramsString, setParamsString] = React.useState('');
  const [errorMessage, setErrorMessage] = React.useState('');

  if (!Array.isArray(params)) {
    params = [params];
  }

  const { queryParams } = useQueryParamsState();
  queryParams.first = 1000; // Set a high limit to ensure all queries are fetched, if we end up with over 1000 queries we'll need a new solution, or if this is too slow...
  const { data: queriesData } = useNotificationQueries(queryParams);
  const selectedQuery = queriesData?.nodes.find(query => query.id === parameterQueryId);

  const paramEditors = (
    <>
      {
        params.length === 0 ? (
          <DetailPanelSection
            key={'new-parameters-section'}
            title={`${t('label.parameters')}`}
            defaultExpanded={false}
            actionButtons={
              <>
                <IconButton
                  onClick={() => {
                    params.push({});
                    onDeleteParam(0, 'this-is-a-hack-to-force-an-update');
                  }}
                  icon={<PlusCircleIcon/>}
                  label={t('button.create')}
                />
              </>
            }
          />
        ) :
        params.map((_, idx) => {
          return (
            <DetailPanelSection
              key={`param-editor-detail-${idx}`}
              title={`${t('label.parameters')}: ${idx + 1}`}
              defaultExpanded={idx === params.length - 1}
              actionButtons={
                <>
                  <IconButton
                    onClick={() => {
                      params.push(params[idx] ?? {});
                      onDeleteParam(idx + 1, 'this-is-a-hack-to-force-an-update');
                    }}
                    disabled={!allowParameterSets}
                    icon={<CopyIcon />}
                    label={t('button.duplicate')}
                  />
                  <IconButton
                    onClick={() => onDeleteParam(idx, null)}
                    icon={<DeleteIcon />}
                    label={t('label.delete')}
                  />
                </>
              }
            >
              <ParameterEditor
                key={`param-editor-${idx}`}
                requiredParams={requiredParams}
                params={params[idx] ?? {}}
                onUpdateParams={(key, value) => onUpdateParams(idx, key, value)}
                onDeleteParam={key => onDeleteParam(idx, key)}
              />
            </DetailPanelSection>
          );
        })
      }
    </>
  );

  const jsonParamsEditor = (
    <DetailPanelSection
      key={`param-editor-detail-json`}
      title={t('label.parameters-as-json')}
      defaultExpanded={true}
    >
      <BufferedTextArea
        value={JSON.stringify(params)}
        onChange={e => {
          setParamsString(e.target.value);
          try {
            JSON.parse(e.target.value);
            setIsJsonEdited(true);
            setErrorMessage('');
          } catch (e) {
            setIsJsonEdited(false);
            if (e instanceof SyntaxError) {
              setErrorMessage(e.message);
            } else {
              alert(e);
            }
          }
        }}
      />
      {isJsonEdited && (
        <IconButton
          onClick={() => {
            try {
              const editedParams: KeyedParams[] = JSON.parse(paramsString);
              editedParams.forEach((param, idx) => {
                if (idx === params.length) {
                  // Allow adding empty sets from JSON
                  params.push({});
                  onDeleteParam(idx, 'this-is-a-hack-to-force-an-update');
                }
                // Iterate through all valid keys
                for (const key of [...Object.keys(param), ...Object.keys(params[idx] || {})]) {
                  if (param[key]) {
                    // If we parsed the key from the JSON input, update it
                    onUpdateParams(idx, key, param[key] || '');
                  } else {
                    // If the key exists on the parameter object but not the JSON, delete it
                    onDeleteParam(idx, key);
                  }
                }
              });
              while (params.length > editedParams.length) {
                onDeleteParam(editedParams.length, null);
              }
            } catch (e) {
              setErrorMessage(`Unable to save new parameters: ${e}`);
            }

            setIsJsonEdited(false);
          }}
          icon={<SaveIcon />}
          label={t('button.save')}
        />
      )}
      {errorMessage ? (
        <Alert
          sx={{ marginTop: 2 }}
          severity="error"
          onClose={() => {
            setErrorMessage('');
          }}
        >
          <AlertTitle>{t('error')}</AlertTitle>
          {errorMessage}
        </Alert>
      ) : null}
    </DetailPanelSection>
  );

  const parameterQuerySelector = (
    <DetailPanelSection
      key={'param-query-selector'}
      title={t('label.parameter-query-select')}
    >
      <Autocomplete
        options={queriesData?.nodes ?? []}
        width="full"
        getOptionLabel={option => option.name}
        onChange={(_, option) => onChangeParameterQuery(option?.id ?? null)}
        value={selectedQuery ? { ...selectedQuery, label: selectedQuery.name } : null}
      />
    </DetailPanelSection>
  );

  return (
    <DetailPanelPortal>
      {paramEditors}
      {jsonParamsEditor}
      {parameterQuerySelector}
    </DetailPanelPortal>
  );
};
