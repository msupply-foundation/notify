import React, { useState } from 'react';
import {
  useDialog,
  DialogButton,
  useTranslation,
  LoadingButton,
  ArrowRightIcon,
  Select,
  ConfigKind,
  FnUtils,
  useNavigate,
  BasicTextInput,
  Box,
} from '@notify-frontend/common';
import { useCreateNotificationConfig } from '../api/hooks/useCreateNotificationConfig';
import { configRoute } from '../navigate';

interface SelectNotificationTypeModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const CreateNotificationModal = ({
  isOpen,
  onClose,
}: SelectNotificationTypeModalProps) => {
  const t = useTranslation('system');
  const navigate = useNavigate();

  const { mutateAsync: create, isLoading } = useCreateNotificationConfig();

  const [kind, setKind] = useState<ConfigKind | ''>('');
  const [title, setTitle] = useState<string>('');

  const { Modal } = useDialog({ isOpen, onClose });

  const isInvalid = !kind;

  return (
    <>
      <Modal
        okButton={
          <LoadingButton
            disabled={isInvalid}
            onClick={() => {
              if (kind) {
                const id = FnUtils.generateUUID();
                create({
                  input: {
                    id: id,
                    kind: kind,
                    title: title,
                  },
                }).then(() => {
                  navigate(configRoute(kind, id));
                });
              }
            }}
            isLoading={isLoading}
            startIcon={<ArrowRightIcon />}
            sx={{ marginLeft: 1 }}
          >
            {t('button.create')}
          </LoadingButton>
        }
        cancelButton={<DialogButton variant="cancel" onClick={onClose} />}
        title={t('label.setup-notification', { type: '' })}
      >
        <Box>
          <Select
            fullWidth
            autoFocus
            required
            label={t('label.select-notification-type')}
            value={kind}
            onChange={e => setKind(e.target.value as ConfigKind)}
            options={[
              {
                label: t(`config-kind.${ConfigKind.ColdChain}`),
                value: ConfigKind.ColdChain,
              },
              {
                label: t(`config-kind.${ConfigKind.Scheduled}`),
                value: ConfigKind.Scheduled,
              },
            ]}
            InputLabelProps={{ shrink: true }}
            sx={{ marginBottom: 2 }}
          />

          <BasicTextInput
            fullWidth
            value={title}
            required
            onChange={e => setTitle(e.target.value)}
            label={t('label.notification-title')}
            InputLabelProps={{ shrink: true }}
          />
        </Box>
      </Modal>
    </>
  );
};
