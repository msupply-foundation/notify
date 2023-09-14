import React from 'react';
import { useTranslation } from '@common/intl';
import {
  AppBarButtonsPortal,
  AppBarContentPortal,
  DataTable,
  LoadingButton,
  NothingHere,
  PlusCircleIcon,
  SearchAndDeleteToolbar,
  TableProvider,
  Typography,
  createTableStore,
  useColumns,
} from '@common/ui';
import { NotificationsModal } from '../Modals/NotificationsModal';
import { useEditModal, useQueryParamsState } from '@common/hooks';
import { NotificationConfigRowFragment, useNotificationConfigs } from '../api';
import { useDeleteNotificationConfig } from '../api/hooks/useDeleteNotificationConfig';

export const ListView = () => {
  const t = useTranslation('system');

  const { filter, queryParams, updatePaginationQuery, updateSortQuery } =
    useQueryParamsState();

  const columns = useColumns<NotificationConfigRowFragment>(
    [
      { key: 'title', label: 'label.title'},
      {
        key: 'kind',
        label: 'label.kind',
        sortable: false,
        Cell: props => (
          <Typography>{t(`config-kind.${props.rowData.kind}`)}</Typography>
        ),
      },
      /*{
        key: 'status', label: 'label.status',
      },*/
      'selection',
    ],
    { sortBy: queryParams.sortBy, onChangeSortBy: updateSortQuery },
    [queryParams.sortBy, updateSortQuery]
  );

  const { data, isError, isLoading } = useNotificationConfigs(queryParams);
  const notificationConfigs = data?.nodes ?? [];

  const { mutateAsync: deleteNotificationConfig, invalidateQueries } =
    useDeleteNotificationConfig();

  const { isOpen, onClose, entity, onOpen } =
    useEditModal<NotificationConfigRowFragment>();

  const pagination = {
    page: queryParams.page,
    offset: queryParams.offset,
    first: queryParams.first,
  };

  return (
    <>
      <NotificationsModal isOpen={isOpen} onClose={onClose} entity={entity} />
      <AppBarButtonsPortal>
        <LoadingButton
          isLoading={false}
          startIcon={<PlusCircleIcon />}
          onClick={() => onOpen()}
        >
          {t('label.new-notification')}
        </LoadingButton>
      </AppBarButtonsPortal>
      <TableProvider createStore={createTableStore}>
        <AppBarContentPortal sx={{ paddingBottom: '16px', flex: 1 }}>
          <SearchAndDeleteToolbar
            data={notificationConfigs}
            filter={filter}
            deleteItem={deleteNotificationConfig}
            invalidateQueries={invalidateQueries}
          />
        </AppBarContentPortal>
        <DataTable
          columns={columns}
          data={notificationConfigs}
          isError={isError}
          isLoading={isLoading}
          onRowClick={onOpen}
          noDataElement={<NothingHere body={t('messages.no-notifications')} />}
          pagination={pagination}
          onChangePage={updatePaginationQuery}
        />
      </TableProvider>
    </>
  );
};
