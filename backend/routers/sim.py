import json

from fastapi import APIRouter, Depends, HTTPException, Request
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from database import get_session
from models import Job, JobStatus
from schemas import JobStatusResponse, SimRequest, SimResponse, TopGearRequest
from services.addon_parser import parse_addon_string
from services.game_data import apply_copy_enchants, upgrade_simc_input
from services.profileset_generator import generate_top_gear_input

router = APIRouter(tags=["sim"])


async def _enqueue_job(request: Request, job_id: str):
    """Enqueue a simulation job using the shared Redis pool."""
    redis = request.app.state.redis
    await redis.enqueue_job("run_simulation", job_id)


@router.post("/api/sim", response_model=SimResponse)
async def create_sim(
    req: SimRequest,
    request: Request,
    session: AsyncSession = Depends(get_session),
):
    simc_input = upgrade_simc_input(req.simc_input) if req.max_upgrade else req.simc_input
    job = Job(
        simc_input=simc_input,
        sim_type=req.sim_type.value,
        iterations=req.iterations,
        fight_style=req.fight_style.value,
        target_error=req.target_error,
    )
    session.add(job)
    await session.commit()
    await session.refresh(job)

    await _enqueue_job(request, job.id)
    return SimResponse(id=job.id, status=job.status.value, created_at=job.created_at)


@router.post("/api/top-gear/sim", response_model=SimResponse)
async def create_top_gear_sim(
    req: TopGearRequest,
    request: Request,
    session: AsyncSession = Depends(get_session),
):
    simc_input = upgrade_simc_input(req.simc_input) if req.max_upgrade else req.simc_input
    parsed = parse_addon_string(simc_input)
    items_by_slot = req.items_by_slot if req.items_by_slot else parsed["items_by_slot"]
    if req.copy_enchants:
        items_by_slot = apply_copy_enchants(items_by_slot)
    try:
        simc_input, combo_count, combo_metadata = generate_top_gear_input(
            base_profile=parsed["base_profile"],
            items_by_slot=items_by_slot,
            selected_items=req.selected_items,
        )
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

    if combo_count == 0:
        raise HTTPException(
            status_code=400,
            detail="No alternative items selected. Select at least one non-equipped item.",
        )

    job = Job(
        simc_input=simc_input,
        sim_type="top_gear",
        iterations=req.iterations,
        fight_style=req.fight_style.value,
        target_error=req.target_error,
        # Store combo metadata separately for result parsing
        combo_metadata_json=json.dumps({
            "_combo_metadata": combo_metadata,
            "_combo_count": combo_count,
        }),
    )
    session.add(job)
    await session.commit()
    await session.refresh(job)

    await _enqueue_job(request, job.id)
    return SimResponse(id=job.id, status=job.status.value, created_at=job.created_at)


@router.get("/api/sim/{job_id}", response_model=JobStatusResponse)
async def get_sim_status(job_id: str, session: AsyncSession = Depends(get_session)):
    result = await session.execute(select(Job).where(Job.id == job_id))
    job = result.scalar_one_or_none()
    if not job:
        raise HTTPException(status_code=404, detail="Job not found")

    parsed_result = None
    if job.status == JobStatus.DONE and job.result_json:
        parsed_result = json.loads(job.result_json)

    progress = 0
    if job.status == JobStatus.RUNNING:
        progress = 50
    elif job.status == JobStatus.DONE:
        progress = 100

    return JobStatusResponse(
        id=job.id,
        status=job.status.value,
        progress=progress,
        result=parsed_result,
        error=job.error_message,
    )


@router.get("/api/sim/{job_id}/raw")
async def get_sim_raw(job_id: str, session: AsyncSession = Depends(get_session)):
    result = await session.execute(select(Job).where(Job.id == job_id))
    job = result.scalar_one_or_none()
    if not job:
        raise HTTPException(status_code=404, detail="Job not found")
    if not job.result_json:
        raise HTTPException(status_code=404, detail="No results available yet")
    return json.loads(job.result_json)
