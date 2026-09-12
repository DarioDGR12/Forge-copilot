import type { ApprovalRequest } from "../types";

type ApprovalModalProps = {
  request: ApprovalRequest;
  onResolve: (allowed: boolean) => void;
};

export function ApprovalModal({ request, onResolve }: ApprovalModalProps) {
  return (
    <div className="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="approve-title">
      <div className="modal">
        <h2 id="approve-title">¿Permitir esta acción?</h2>
        <p className="muted">
          Forge quiere usar <strong>{request.name}</strong> en tu sistema.
        </p>
        {request.sensitive ? (
          <p className="warn">
            Esta ruta o comando es sensible. Solo continúa si reconoces exactamente qué va a hacer.
          </p>
        ) : null}
        <pre className="reason">{request.reason}</pre>
        {request.arguments ? <pre className="args">{pretty(request.arguments)}</pre> : null}
        <div className="modal-actions">
          <button type="button" className="ghost" onClick={() => onResolve(false)}>
            Denegar
          </button>
          <button
            type="button"
            className={request.sensitive ? "danger-solid" : "primary"}
            onClick={() => onResolve(true)}
          >
            {request.sensitive ? "Permitir de todos modos" : "Permitir"}
          </button>
        </div>
      </div>
    </div>
  );
}

function pretty(raw: string): string {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
}
