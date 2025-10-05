import React, { FC, useCallback } from 'react';
import {
  useColumns,
  useTranslation,
  StringUtils,
  useEditModal,
  EditIcon,
  DataTable,
  TableProvider,
  createTableStore,
  LoadingButton,
  Box,
} from '@notify-frontend/common';
import { NotificationQuerySelectionModal } from './NotificationQuerySelectionModal';
import { NotificationQueryRowFragment } from '../../Queries/api';

type QueryListProps = {
  allQueries: NotificationQueryRowFragment[];
  selectedQueryIds: string[];
  requiredQueryIds: string[];
  setSelection: (input: {
    notificationQueryIds: string[];
    requiredParameters: string[];
  }) => void;
  setRequiredQueryIds: (ids: string[]) => void;
  isLoading: boolean;
};

export const SqlQuerySelector: FC<QueryListProps> = ({
  allQueries,
  selectedQueryIds,
  requiredQueryIds,
  setSelection,
  setRequiredQueryIds,
  isLoading,
}) => {
  const t = useTranslation('system');

  const { isOpen, onClose, onOpen } = useEditModal();

  const onSetRequired = useCallback(
    (id: string, required: boolean) => {
      let newRequiredIds = [...requiredQueryIds];
      if (required) {
        if (!newRequiredIds.includes(id)) {
          newRequiredIds.push(id);
        }
      } else {
        newRequiredIds = newRequiredIds.filter(rid => rid !== id);
      }
      setRequiredQueryIds(newRequiredIds);
    },
    [requiredQueryIds]
  );

  const columns = useColumns<
    NotificationQueryRowFragment & { required: boolean }
  >(
    [
      {
        key: 'referenceName',
        label: 'label.reference-name',
        width: 200,
        sortable: false,
      },
      {
        key: 'name',
        label: 'label.name',
        width: 150,
        sortable: false,
      },
      {
        key: 'query',
        label: 'label.query',
        width: 150,
        sortable: false,
        accessor: ({ rowData }) => StringUtils.ellipsis(rowData?.query, 50),
      },
      {
        key: 'requiredParameters',
        label: 'label.parameters',
        sortable: false,
        accessor: ({ rowData }) => rowData?.requiredParameters.join(', '),
      },
      {
        key: 'required',
        label: 'label.required',
        width: 100,
        sortable: false,
        Cell: ({ rowData }) => (
          <input
            type="checkbox"
            checked={rowData?.required || false}
            onChange={e => onSetRequired(rowData.id, e.target.checked)}
          />
        ),
      },
    ],
    {},
    [onSetRequired]
  );

  const selectedQueries = (allQueries ?? [])
    .filter(q => selectedQueryIds.includes(q.id))
    .map(q => ({
      ...q,
      required: requiredQueryIds?.includes(q.id) ?? false,
    }));

  return (
    <>
      <NotificationQuerySelectionModal
        sqlQueries={allQueries}
        initialSelectedIds={selectedQueryIds}
        isOpen={isOpen}
        onClose={onClose}
        setSelection={setSelection}
      />
      <TableProvider createStore={createTableStore}>
        <DataTable
          isDisabled={false}
          isLoading={isLoading}
          columns={columns}
          data={selectedQueries}
          noDataMessage={t('message.no-queries-selected')}
        />
      </TableProvider>
      <Box padding={2}>
        <LoadingButton
          disabled={false}
          onClick={onOpen}
          isLoading={false}
          startIcon={<EditIcon />}
        >
          {t('label.select-queries')}
        </LoadingButton>
      </Box>
    </>
  );
};
