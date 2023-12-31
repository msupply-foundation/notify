import React from 'react';
import {
  BasicTextInput,
  BufferedTextArea,
  Grid,
  useTranslation,
} from '@notify-frontend/common';
import { DraftRecipientList } from './types';

type RecipientListEditFormProps = {
  draft: DraftRecipientList;
  onUpdate: (patch: Partial<DraftRecipientList>) => void;
};

export const RecipientListEditForm = ({
  draft,
  onUpdate,
}: RecipientListEditFormProps) => {
  const t = useTranslation('system');

  return (
    <Grid flexDirection="column" display="flex" gap={2}>
      <BasicTextInput
        autoFocus
        required
        value={draft.name}
        helperText={t('helper-text.recipient-list-name')}
        onChange={e => onUpdate({ name: e.target.value })}
        label={t('label.name')}
        InputLabelProps={{ shrink: true }}
      />

      <BufferedTextArea
        value={draft.description}
        onChange={e => onUpdate({ description: e.target.value })}
        label={t('label.description')}
        InputProps={{ sx: { backgroundColor: 'background.menu' } }}
        InputLabelProps={{ shrink: true }}
      />
    </Grid>
  );
};
