import { useGql, useQuery } from '@notify-frontend/common';
import { getSdk } from './../operations.generated';

export type SensorData = {
  id: string;
  store_name: string;
  location_name: string;
  sensor_name: string;
};

export function sensorDisplayName(sensor: SensorData): string {
  if (!sensor.location_name) {
    return `${sensor.store_name} - ${sensor.sensor_name}`;
  }
  return `${sensor.store_name} - ${sensor.location_name} - ${sensor.sensor_name}`;
}

export const useColdChainSensors = () => {
  const { client } = useGql();
  const sdk = getSdk(client);

  const cacheKeys = ['COLDCHAIN_SENSORS'];

  return useQuery(cacheKeys, async () => {
    const sensorQuery =
      "SELECT sn.id as id, s.name as store_name,coalesce(l.description, '') as location_name, sn.name as sensor_name FROM SENSOR sn JOIN store s ON sn.storeid = s.id LEFT JOIN location l on sn.locationid = l.id WHERE sn.is_active = true ORDER BY 2,3,4 LIMIT 1000";
    const response = await sdk.runSqlQueryWithParameters({
      sqlQuery: sensorQuery,
      params: '{}',
    });

    if (!response) {
      return [];
    }else{
      const responseType = response.runSqlQueryWithParameters.__typename;
      
      if (responseType == "NodeError"){
        return [];
      }else{
        const sensors: SensorData[] = JSON.parse(response.runSqlQueryWithParameters.results);
        return sensors;
      }
    }
  });
};
