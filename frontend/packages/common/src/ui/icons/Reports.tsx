import React from 'react';
import SvgIcon, { SvgIconProps } from '@mui/material/SvgIcon';

export const ReportsIcon = (props: SvgIconProps): JSX.Element => {
  return (
    <SvgIcon {...props} viewBox="0 0 20 20">
      <path d="M6.334 1.595c.422-.185.913.008 1.097.43.184.422-.008.913-.43 1.097C3.91 4.47 2.1 7.713 2.575 11.05c.473 3.337 3.115 5.948 6.457 6.383 3.342.436 6.564-1.412 7.876-4.516.18-.424.668-.622 1.092-.443.424.18.623.668.443 1.092-1.604 3.794-5.542 6.052-9.627 5.52-4.084-.532-7.313-3.724-7.891-7.802-.579-4.078 1.634-8.042 5.41-9.69zm3.667-.762c2.431 0 4.763.966 6.482 2.685 1.719 1.72 2.684 4.05 2.684 6.482 0 .46-.373.833-.833.833h-8.333c-.46 0-.834-.373-.834-.833V1.667c0-.46.374-.834.834-.834zm.833 1.713v6.62h6.62c-.188-1.682-.941-3.26-2.15-4.47-1.209-1.208-2.787-1.962-4.47-2.15z" />{' '}
    </SvgIcon>
  );
};
