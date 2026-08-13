import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { AppConfig } from "@/types";

import { LocalAsrSettings } from "./LocalAsrSettings";
import { SettingsFieldList } from "./SettingsFieldList";
import { settingsSchema, type SettingsSaveMode } from "./settings-schema";

type ModelSettingsProps = {
  appConfig: AppConfig | null;
  updateConfig: (patch: Partial<AppConfig>, saveMode: SettingsSaveMode) => void;
  onConfigBlur: () => void;
  onModelChanged: () => void;
};

export function ModelSettings({
  appConfig,
  updateConfig,
  onConfigBlur,
  onModelChanged,
}: ModelSettingsProps) {
  if (!appConfig) return null;

  return settingsSchema.models.map((section) => {
    return (
      <Card key={section.id}>
        <CardHeader>
          <CardTitle>{section.title}</CardTitle>
          <CardDescription>{section.description}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4">
            <SettingsFieldList
              section={section}
              config={appConfig}
              context={{ audioDevices: [] }}
              onChange={updateConfig}
              onBlur={onConfigBlur}
              renderSlot={(slot, saveMode) =>
                slot === "local-asr-settings" ? (
                  <LocalAsrSettings
                    config={appConfig}
                    onChange={(config) => updateConfig(config, saveMode)}
                    onModelChanged={onModelChanged}
                  />
                ) : null
              }
            />
          </div>
        </CardContent>
      </Card>
    );
  });
}
