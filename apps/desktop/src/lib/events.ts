import { useEffect, useState, useRef } from "react";
import { listen, Event } from "@tauri-apps/api/event";

export interface AgentEvent<T = unknown> {
  id: string;
  type: string;
  payload: T;
}

export function useAgentEventStream<T = unknown>(topic: string = "agent://app") {
  const [events, setEvents] = useState<AgentEvent<T>[]>([]);
  const batchRef = useRef<AgentEvent<T>[]>([]);
  const frameRef = useRef<number>(0);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setup = async () => {
      unlisten = await listen<AgentEvent<T>>(topic, (event: Event<AgentEvent<T>>) => {
        batchRef.current.push(event.payload);
        
        if (!frameRef.current) {
          frameRef.current = requestAnimationFrame(() => {
            setEvents((prev) => [...prev, ...batchRef.current]);
            batchRef.current = [];
            frameRef.current = 0;
          });
        }
      });
    };

    setup();

    return () => {
      if (unlisten) unlisten();
      if (frameRef.current) {
        cancelAnimationFrame(frameRef.current);
      }
    };
  }, [topic]);

  return events;
}
