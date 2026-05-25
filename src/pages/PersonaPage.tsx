import React from "react";
import { PencilIcon, PlusIcon, Trash2Icon } from "lucide-react";
import * as LucideIcons from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { Persona } from "@/types";

type PersonaPageProps = {
  personas: Persona[];
  status: string;
  onCreatePersona: () => void;
  onEditPersona: (persona: Persona) => void;
  onDeletePersona: (id: string) => void;
  onSetDefaultPersona: (personaId: string) => void;
};

export function PersonaPage({
  personas,
  status,
  onCreatePersona,
  onEditPersona,
  onDeletePersona,
  onSetDefaultPersona,
}: PersonaPageProps) {
  const renderPersonaIcon = (iconName: string) => {
    const IconComponent = LucideIcons[iconName as keyof typeof LucideIcons];
    if (IconComponent) {
      return <IconComponent className="size-5 shrink-0" aria-hidden="true" />;
    }
    return null;
  };

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <div>
            <CardTitle className="text-2xl">人格管理</CardTitle>
            <CardDescription className="mt-2">
              管理人格，设置默认人格。所有人格都可以编辑和删除。
            </CardDescription>
          </div>
          <CardAction>
            <Button type="button" size="sm" onClick={onCreatePersona}>
              <PlusIcon className="size-4" aria-hidden="true" />
              新建人格
            </Button>
          </CardAction>
        </CardHeader>

        <CardContent className="space-y-5">
          <div className="grid gap-3">
            {personas.length > 0 ? (
              personas.map((persona) => (
                <section
                  key={persona.id}
                  className="rounded-lg border bg-background p-4"
                >
                  <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                    <div className="min-w-0 flex-1 flex items-start gap-3">
                      <div className="mt-0.5">
                        {renderPersonaIcon(persona.icon)}
                      </div>
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <p className="text-sm font-semibold">
                            {persona.name}
                          </p>
                          {persona.is_default ? (
                            <span className="inline-flex h-6 items-center rounded-md border bg-background px-2 text-xs font-medium">
                              默认
                            </span>
                          ) : null}
                        </div>
                        <p className="mt-1 text-sm leading-6 text-muted-foreground">
                          {persona.description}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {!persona.is_default ? (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() => onSetDefaultPersona(persona.id)}
                        >
                          设为默认
                        </Button>
                      ) : null}
                      <Button
                        type="button"
                        variant="outline"
                        size="icon"
                        onClick={() => onEditPersona(persona)}
                        aria-label={`编辑 ${persona.name}`}
                      >
                        <PencilIcon className="size-4" aria-hidden="true" />
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        size="icon"
                        onClick={() => onDeletePersona(persona.id)}
                        aria-label={`删除 ${persona.name}`}
                      >
                        <Trash2Icon className="size-4" aria-hidden="true" />
                      </Button>
                    </div>
                  </div>
                </section>
              ))
            ) : (
              <section className="rounded-lg border border-dashed bg-muted/20 p-5 text-sm leading-6 text-muted-foreground">
                暂无人格。可以新建人格来定义自己的文本整理风格。
              </section>
            )}
          </div>

          <div className="flex flex-col gap-2 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
            <p className="text-sm leading-6 text-muted-foreground">
              {status}
            </p>
            <span className="inline-flex h-8 w-fit items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
              共 {personas.length} 个人格
            </span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
