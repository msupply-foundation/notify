import React from 'react';
import {
  Box,
  DeleteIcon,
  DetailPanelPortal,
  PanelLabel,
  PanelRow,
} from '@common/ui';
import {
  BufferedTextInput,
  DetailPanelSection,
  IconButton,
} from '@common/components';
import { KeyedParams } from '@common/utils';
import { useTranslation } from '@common/intl';

export interface ParamsPanelProps {
  requiredParams: string[];
  params: KeyedParams;
  onUpdateParams: (key: string, value: string) => void;
  onDeleteParam: (key: string) => void;
}

export const NotificationDetailPanel = ({
  requiredParams,
  params,
  onUpdateParams,
  onDeleteParam,
}: ParamsPanelProps) => {
  const t = useTranslation('system');

  const allParams = [...new Set(requiredParams.concat(Object.keys(params)))];

  const paramEditor = (
    <DetailPanelSection title={t('label.parameters')}>
      {allParams.map(param => {
        return (
          <Box key={`param-row-${param}`} paddingBottom={1}>
            <PanelRow>
              <PanelLabel key={`param-label-${param}`}>{param}</PanelLabel>
            </PanelRow>
            <PanelRow>
              <BufferedTextInput
                key={`param-value-${param}`}
                InputProps={{
                  sx: {
                    backgroundColor: 'background.white',
                  },
                }}
                value={params[param ?? '']}
                onChange={e => onUpdateParams(param ?? '', e.target.value)}
              />
              {
                // if param is not required allow it to be removed
                !requiredParams.includes(param) && (
                  <IconButton
                    onClick={() => onDeleteParam(param ?? '')}
                    icon={<DeleteIcon />}
                    label={t('label.delete')}
                  />
                )
              }
            </PanelRow>
          </Box>
        );
      })}
    </DetailPanelSection>
  );

  return <DetailPanelPortal>{paramEditor}</DetailPanelPortal>;
};